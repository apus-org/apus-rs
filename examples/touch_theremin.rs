#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;
use panic_probe as _;

use apus::board::{defaults::SPI1_FREQUENCY, resources::Resources};
use apus::bsp::{
    buzzer::Buzzer,
    delay::BusyDelay,
    display::{self, DisplaySize, DISPLAY_HEIGHT, DISPLAY_WIDTH},
    touch::{self, Touch},
};
use apus::peripherals::spi::spi1;
use apus::prelude::*;

use apus::hal::pac;
use embedded_hal_bus::spi::ExclusiveDevice;

mod common;

// C-major pentatonic, C4..A6.
// Note frequencies are log-spaced
const NOTES: [u32; 15] = [
    262, 294, 330, 392, 440, 523, 587, 659, 784, 880, 1047, 1175, 1319, 1568, 1760,
];

const TONE_LENGTH_MS: u32 = 20;

// pitch bar geometry
const BAR_X: i32 = 0;
const BAR_WIDTH: u32 = 40;
const BAR_COLOR: Rgb565 = Rgb565::CYAN;
const BG_COLOR: Rgb565 = Rgb565::BLACK;

#[entry]
fn main() -> ! {
    defmt::info!("APUS-RS example: touch_theremin");

    let p = pac::Peripherals::take().unwrap();

    let mut res = Resources::new(p);

    let sys_clock_hz = res.rcc.clocks.sysclk().0;

    let mut display = display::init(res.display, BusyDelay::new(sys_clock_hz));

    let (spi_bus, cs_pin) =
        spi1::init(res.peripherals.spi1, res.spi1, SPI1_FREQUENCY, &mut res.rcc);
    let spi_dev = ExclusiveDevice::new(spi_bus, cs_pin, BusyDelay::new(sys_clock_hz)).unwrap();
    let mut touch = Touch::new(
        spi_dev,
        res.touch,
        touch::DEFAULT_THRESHOLD,
        DisplaySize {
            x: DISPLAY_WIDTH,
            y: DISPLAY_HEIGHT,
        },
    );

    let mut buzzer = Buzzer::new(res.buzzer);
    let mut delay = BusyDelay::new(sys_clock_hz);

    // calibration
    defmt::info!("touch each marker as it appears");
    touch.calibrate_from_display(&mut display).unwrap();
    defmt::info!("calibration complete - touch to play");

    let mut prev_h = 0u16;
    loop {
        match touch.read() {
            Ok(Some(tp)) => {
                // top of screen = high pitch
                let y = tp.y.min(DISPLAY_HEIGHT - 1);
                let idx =
                    ((DISPLAY_HEIGHT - 1 - y) as usize * NOTES.len()) / DISPLAY_HEIGHT as usize;
                let idx = idx.min(NOTES.len() - 1);

                let h = (((idx + 1) * DISPLAY_HEIGHT as usize) / NOTES.len()) as u16;
                common::vbar_draw_delta(
                    &mut display,
                    prev_h,
                    h,
                    BAR_COLOR,
                    BG_COLOR,
                    BAR_X,
                    BAR_WIDTH,
                );
                prev_h = h;

                common::play_tone(&mut buzzer, &mut delay, NOTES[idx], TONE_LENGTH_MS);
            }
            _ => {
                common::vbar_draw_delta(
                    &mut display,
                    prev_h,
                    0,
                    BAR_COLOR,
                    BG_COLOR,
                    BAR_X,
                    BAR_WIDTH,
                );
                prev_h = 0;
                buzzer.off();
                delay.delay_ms(TONE_LENGTH_MS);
            }
        }
    }
}
