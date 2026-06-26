use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{DrawTarget, Point, RgbColor, Size};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use stm32f0xx_hal::gpio::{gpioe, Output};
use tsc2046::Tsc2046;

use crate::bsp::delay::BusyDelay;
use crate::bsp::display::DisplaySize;
use crate::peripherals::spi::spi1::Spi1;

pub const DEFAULT_THRESHOLD: f32 = 1.0; // TODO: find optimal value

/// SPI device type for the touch controller
pub type ApusTouchDevice = embedded_hal_bus::spi::ExclusiveDevice<
    Spi1,
    gpioe::PE12<Output<stm32f0xx_hal::gpio::PushPull>>,
    BusyDelay,
>;

/// Concrete configured touch driver type used by the board.
pub type ApusTouch = Tsc2046<ApusTouchDevice>;

#[derive(Clone, Copy, Debug)]
pub struct TouchPoint2D {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct TouchPoint3D {
    pub x: u16,
    pub y: u16,
    /// The pressure value of the touch point, ranging from 0.0 (max pressure) to the set touch threshold.
    pub z: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct TouchScale {
    x: f32,
    y: f32,
}

pub type TouchOffset = TouchPoint2D;

#[derive(Clone, Copy, Debug)]
pub struct TouchCalibrationData {
    offset: TouchOffset,
    scale: TouchScale,
}

impl TouchCalibrationData {
    /// Length of the byte representation produced by [`Self::to_bytes`].
    pub const SERIALIZED_LEN: usize = 12;

    /// Serialize to a fixed byte array (offsets as u16, scales as f32, little-endian).
    /// Persisting these bytes is the caller's concern.
    pub fn to_bytes(&self) -> [u8; Self::SERIALIZED_LEN] {
        let mut buf = [0u8; Self::SERIALIZED_LEN];
        buf[0..2].copy_from_slice(&self.offset.x.to_le_bytes());
        buf[2..4].copy_from_slice(&self.offset.y.to_le_bytes());
        buf[4..8].copy_from_slice(&self.scale.x.to_le_bytes());
        buf[8..12].copy_from_slice(&self.scale.y.to_le_bytes());
        buf
    }

    /// Reconstruct from bytes produced by [`Self::to_bytes`].
    pub fn from_bytes(bytes: &[u8; Self::SERIALIZED_LEN]) -> Self {
        Self {
            offset: TouchOffset {
                x: u16::from_le_bytes([bytes[0], bytes[1]]),
                y: u16::from_le_bytes([bytes[2], bytes[3]]),
            },
            scale: TouchScale {
                x: f32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
                y: f32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
            },
        }
    }
}

#[derive(Debug)]
pub enum TouchError {
    NotCalibrated,
    SpiError,
}

pub type TouchResult = Result<Option<TouchPoint3D>, TouchError>;

pub struct Touch {
    touch: ApusTouch,
    size: DisplaySize,
    calibration_data: Option<TouchCalibrationData>,
}

impl Touch {
    pub fn new(device: ApusTouchDevice, touch_threshold: f32, size: DisplaySize) -> Self {
        let touch = Tsc2046::new(device, true, touch_threshold).unwrap();
        let calibration_data = None;
        Self {
            touch,
            size,
            calibration_data,
        }
    }

    pub fn read_raw(
        &mut self,
    ) -> Result<Option<tsc2046::TouchPoint>, <ApusTouchDevice as embedded_hal::spi::ErrorType>::Error>
    {
        self.touch.get_touch()
    }

    pub fn read(&mut self) -> TouchResult {
        // Read raw touch data
        let raw_touch = self.read_raw().map_err(|_| TouchError::SpiError)?;

        // Check if calibration data is available
        let calibration = self
            .calibration_data
            .as_mut()
            .ok_or(TouchError::NotCalibrated)?;

        // If no touch detected, return None
        let raw_point = match raw_touch {
            // pressure value from touch display has inverted scale
            // meaning that low values represent higher pressure.
            // 0.0 is the maximum pressure value but can never actually be reached
            // in real measurements, but only in erroneous measurements.
            // Therefore, 0.0 is filtered out.
            Some(tp) if tp.z > 0.0 => tp,
            _ => return Ok(None),
        };

        // Apply calibration transformation using u32 with clamping
        let calibrated_x = {
            let raw_x = raw_point.x.saturating_sub(calibration.offset.x) as u32;
            let scaled_x = (raw_x as f32 * calibration.scale.x) as u32;
            scaled_x.min(self.size.x as u32) as u16
        };

        let calibrated_y = {
            let raw_y = raw_point.y.saturating_sub(calibration.offset.y) as u32;
            let scaled_y = (raw_y as f32 * calibration.scale.y) as u32;
            let inverted_y = (self.size.y as u32).saturating_sub(scaled_y);
            inverted_y.min(self.size.y as u32) as u16
        };

        Ok(Some(TouchPoint3D {
            x: calibrated_x,
            y: calibrated_y,
            z: raw_point.z,
        }))
    }

    pub fn set_touch_threshold(&mut self, touch_threshold: f32) {
        self.touch.set_touch_threshold(touch_threshold);
    }

    pub fn calibrate_from_display(
        &mut self,
        display: &mut impl DrawTarget<Color = Rgb565>,
    ) -> TouchCalibrationData {
        const MARKER_SIZE: u16 = 20;

        // Calibrate each corner
        let top_left = self.calibrate_corner(display, 0, 0, MARKER_SIZE);
        let top_right = self.calibrate_corner(display, self.size.x - MARKER_SIZE, 0, MARKER_SIZE);
        let bottom_left = self.calibrate_corner(display, 0, self.size.y - MARKER_SIZE, MARKER_SIZE);
        let bottom_right = self.calibrate_corner(
            display,
            self.size.x - MARKER_SIZE,
            self.size.y - MARKER_SIZE,
            MARKER_SIZE,
        );

        // Calculate average offset from all four corners
        let x_offset = ((top_left.x as u32 + bottom_left.x as u32) / 2) as u16;
        let y_offset = ((bottom_left.y as u32 + bottom_right.y as u32) / 2) as u16;

        // Calculate scale
        // Distance in touch coordinates between calibration points (average from both sides)
        let touch_x_span_top = (top_right.x as i32 - top_left.x as i32).abs() as f32;
        let touch_x_span_bottom = (bottom_right.x as i32 - bottom_left.x as i32).abs() as f32;
        let touch_x_span = (touch_x_span_top + touch_x_span_bottom) / 2.0;

        let touch_y_span_left = (bottom_left.y as i32 - top_left.y as i32).abs() as f32;
        let touch_y_span_right = (bottom_right.y as i32 - top_right.y as i32).abs() as f32;
        let touch_y_span = (touch_y_span_left + touch_y_span_right) / 2.0;

        // Calculate scale factors
        let x_scale = self.size.x as f32 / touch_x_span;
        let y_scale = self.size.y as f32 / touch_y_span;

        TouchCalibrationData {
            offset: TouchOffset {
                x: x_offset,
                y: y_offset,
            },
            scale: TouchScale {
                x: x_scale,
                y: y_scale,
            },
        }
    }

    pub fn calibrate_from_data(&mut self, data: TouchCalibrationData) {
        self.calibration_data = Some(data);
    }

    // Helper function to calibrate a single corner by drawing a marker,
    // sampling touch readings, averaging them, and clearing the marker.
    fn calibrate_corner(
        &mut self,
        display: &mut impl DrawTarget<Color = Rgb565>,
        x: u16,
        y: u16,
        marker_size: u16,
    ) -> TouchPoint2D {
        const MARKER_COLOR: Rgb565 = Rgb565::WHITE;
        const CLEAR_COLOR: Rgb565 = Rgb565::BLACK;
        const STYLE: PrimitiveStyle<Rgb565> = PrimitiveStyle::with_fill(MARKER_COLOR);
        const NUM_TOUCH_SAMPLES: usize = 10; // Number of consecutive touch readings required
        const NUM_RELEASE_SAMPLES: usize = 5; // Number of consecutive "no touch" readings required

        // Draw marker rectangle at corner
        let rect = Rectangle::new(
            Point::new(x as i32, y as i32),
            Size::new(marker_size as u32, marker_size as u32),
        );
        let _ = rect.draw_styled(&STYLE, display);

        // Read multiple samples and average them, restarting if touch is released
        let mut sum_x = 0u32;
        let mut sum_y = 0u32;
        let mut count = 0;

        while count < NUM_TOUCH_SAMPLES {
            if let Ok(Some(tp)) = self.read_raw() {
                sum_x += tp.x as u32;
                sum_y += tp.y as u32;
                count += 1;
            } else {
                // Touch was released or error occurred, start over
                sum_x = 0;
                sum_y = 0;
                count = 0;
            }
        }

        // Wait for touch to be released for specified consecutive readings
        let mut release_count = 0;
        while release_count < NUM_RELEASE_SAMPLES {
            if let Ok(None) = self.read_raw() {
                release_count += 1;
            } else {
                release_count = 0;
            }
        }

        // Clear the marker
        let _ = rect.draw_styled(&PrimitiveStyle::with_fill(CLEAR_COLOR), display);

        // Return averaged touch point
        let avg_x = (sum_x / NUM_TOUCH_SAMPLES as u32) as u16;
        let avg_y = (sum_y / NUM_TOUCH_SAMPLES as u32) as u16;

        TouchPoint2D { x: avg_x, y: avg_y }
    }
}
