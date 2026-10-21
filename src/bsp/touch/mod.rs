use cortex_m::interrupt;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{DrawTarget, Point, RgbColor, Size};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use embedded_hal::digital::InputPin;
use stm32f0xx_hal::gpio::{gpiof, Floating, Input, PullDown, PullUp};
use tsc2046::Tsc2046;

use crate::bsp::delay::BusyDelay;
use crate::bsp::display::DisplaySize;
use crate::peripherals::spi::spi1::{Spi1, Spi1Cs};

pub const DEFAULT_THRESHOLD: f32 = 1.0; // TODO: find optimal value

/// Edge length of the calibration markers, which are pressed at their centre
const MARKER_SIZE: u16 = 20;

/// SPI device type for the touch controller
pub type ApusTouchDevice = embedded_hal_bus::spi::ExclusiveDevice<Spi1, Spi1Cs, BusyDelay>;

/// Concrete configured touch driver type used by the board.
pub type ApusTouch = Tsc2046<ApusTouchDevice>;

/// Error of the SPI device the touch controller sits on
pub type TouchSpiError = <ApusTouchDevice as embedded_hal::spi::ErrorType>::Error;

/// Status lines of the touch controller, unconfigured
pub struct TouchPins {
    /// PF2 · pen interrupt, active low
    pub penirq: gpiof::PF2<Input<Floating>>,
    /// PF3 · high while a conversion runs
    pub busy: gpiof::PF3<Input<Floating>>,
}

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
    Spi(TouchSpiError),
}

pub type TouchResult = Result<Option<TouchPoint3D>, TouchError>;

pub struct Touch {
    touch: ApusTouch,
    penirq: gpiof::PF2<Input<PullUp>>,
    busy: gpiof::PF3<Input<PullDown>>,
    size: DisplaySize,
    calibration_data: Option<TouchCalibrationData>,
}

impl Touch {
    pub fn new(
        device: ApusTouchDevice,
        pins: TouchPins,
        touch_threshold: f32,
        size: DisplaySize,
    ) -> Self {
        let touch = Tsc2046::new(device, true, touch_threshold).unwrap();
        let (penirq, busy) = interrupt::free(|cs| {
            (
                pins.penirq.into_pull_up_input(cs),
                pins.busy.into_pull_down_input(cs),
            )
        });
        let calibration_data = None;
        Self {
            touch,
            penirq,
            busy,
            size,
            calibration_data,
        }
    }

    /// Whether the panel is pressed, from the pen interrupt line without an SPI transfer
    pub fn touched(&mut self) -> bool {
        self.penirq.is_low().unwrap_or(false)
    }

    /// Whether the controller is converting
    pub fn busy(&mut self) -> bool {
        self.busy.is_high().unwrap_or(false)
    }

    pub fn read_raw(&mut self) -> Result<Option<tsc2046::TouchPoint>, TouchSpiError> {
        self.touch.get_touch()
    }

    pub fn read(&mut self) -> TouchResult {
        let calibration = self.calibration_data.ok_or(TouchError::NotCalibrated)?;

        let raw_point = match self.read_raw().map_err(TouchError::Spi)? {
            // pressure value from touch display has inverted scale
            // meaning that low values represent higher pressure.
            // 0.0 is the maximum pressure value but can never actually be reached
            // in real measurements, but only in erroneous measurements.
            // Therefore, 0.0 is filtered out.
            Some(point) if point.z > 0.0 => point,
            _ => return Ok(None),
        };

        // the offsets are the readings at the centres of the left and bottom markers
        let marker_center = MARKER_SIZE / 2;
        let left_center = f32::from(marker_center);
        let bottom_center = f32::from(self.size.y - marker_center);

        let distance_x = scale_distance(raw_point.x, calibration.offset.x, calibration.scale.x);
        let distance_y = scale_distance(raw_point.y, calibration.offset.y, calibration.scale.y);
        let x = clamp_to_axis(left_center + distance_x, self.size.x);
        let y = clamp_to_axis(bottom_center - distance_y, self.size.y);

        Ok(Some(TouchPoint3D {
            x,
            y,
            z: raw_point.z,
        }))
    }

    pub fn set_touch_threshold(&mut self, touch_threshold: f32) {
        self.touch.set_touch_threshold(touch_threshold);
    }

    /// Shows a marker in each corner to be pressed, then applies and returns the calibration
    pub fn calibrate_from_display(
        &mut self,
        display: &mut impl DrawTarget<Color = Rgb565>,
    ) -> Result<TouchCalibrationData, TouchError> {
        let left = 0;
        let top = 0;
        let right = self.size.x - MARKER_SIZE;
        let bottom = self.size.y - MARKER_SIZE;
        let marker = |x: u16, y: u16| {
            Rectangle::new(
                Point::new(i32::from(x), i32::from(y)),
                Size::new(u32::from(MARKER_SIZE), u32::from(MARKER_SIZE)),
            )
        };

        let top_left = self.calibrate_corner(display, marker(left, top))?;
        let top_right = self.calibrate_corner(display, marker(right, top))?;
        let bottom_left = self.calibrate_corner(display, marker(left, bottom))?;
        let bottom_right = self.calibrate_corner(display, marker(right, bottom))?;

        let span_x = mean_span(top_left.x, top_right.x, bottom_left.x, bottom_right.x);
        let span_y = mean_span(top_left.y, bottom_left.y, top_right.y, bottom_right.y);

        let data = TouchCalibrationData {
            offset: TouchOffset {
                x: mean(top_left.x, bottom_left.x),
                y: mean(bottom_left.y, bottom_right.y),
            },
            // the readings span the distance between the marker centres
            scale: TouchScale {
                x: f32::from(right - left) / span_x,
                y: f32::from(bottom - top) / span_y,
            },
        };
        self.calibrate_from_data(data);

        Ok(data)
    }

    pub fn calibrate_from_data(&mut self, data: TouchCalibrationData) {
        self.calibration_data = Some(data);
    }

    /// Draws `marker`, averages the readings while it is pressed, waits for the release and
    /// clears it again
    fn calibrate_corner(
        &mut self,
        display: &mut impl DrawTarget<Color = Rgb565>,
        marker: Rectangle,
    ) -> Result<TouchPoint2D, TouchError> {
        const MARKER_COLOR: Rgb565 = Rgb565::WHITE;
        const CLEAR_COLOR: Rgb565 = Rgb565::BLACK;
        // consecutive readings averaged while the marker is pressed
        const TOUCH_SAMPLES: u32 = 10;
        // consecutive readings without touch that count as released
        const RELEASE_SAMPLES: u32 = 5;

        let _ = marker.draw_styled(&PrimitiveStyle::with_fill(MARKER_COLOR), display);

        let point = loop {
            if let Some(point) = self.mean_raw(TOUCH_SAMPLES).map_err(TouchError::Spi)? {
                break point;
            }
        };
        while !self
            .untouched_raw(RELEASE_SAMPLES)
            .map_err(TouchError::Spi)?
        {}

        let _ = marker.draw_styled(&PrimitiveStyle::with_fill(CLEAR_COLOR), display);

        Ok(point)
    }

    /// Mean of `samples` raw readings in a row, or `None` if one of them finds no touch
    fn mean_raw(&mut self, samples: u32) -> Result<Option<TouchPoint2D>, TouchSpiError> {
        let mut sum_x = 0;
        let mut sum_y = 0;
        for _ in 0..samples {
            let Some(point) = self.read_raw()? else {
                return Ok(None);
            };
            sum_x += u32::from(point.x);
            sum_y += u32::from(point.y);
        }

        Ok(Some(TouchPoint2D {
            x: (sum_x / samples) as u16,
            y: (sum_y / samples) as u16,
        }))
    }

    /// Whether `samples` raw readings in a row all find no touch
    fn untouched_raw(&mut self, samples: u32) -> Result<bool, TouchSpiError> {
        for _ in 0..samples {
            if self.read_raw()?.is_some() {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// Signed distance of a reading from `offset`, in pixels
fn scale_distance(raw: u16, offset: u16, scale: f32) -> f32 {
    (f32::from(raw) - f32::from(offset)) * scale
}

/// `position` as a pixel of an axis `size` pixels long
fn clamp_to_axis(position: f32, size: u16) -> u16 {
    let last_pixel = f32::from(size - 1);
    position.clamp(0.0, last_pixel) as u16
}

fn mean(a: u16, b: u16) -> u16 {
    ((u32::from(a) + u32::from(b)) / 2) as u16
}

/// Mean distance along one axis between the readings of two pairs of corners
fn mean_span(a_start: u16, a_end: u16, b_start: u16, b_end: u16) -> f32 {
    (f32::from(a_start.abs_diff(a_end)) + f32::from(b_start.abs_diff(b_end))) / 2.0
}
