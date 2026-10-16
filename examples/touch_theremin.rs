#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus::bsp::{
    buzzer::Buzzer,
    delay::BusyDelay,
    display::{self, ApusDisplayPins, DisplaySize, DISPLAY_HEIGHT, DISPLAY_WIDTH},
    touch::{self, Touch},
};
use apus::peripherals::spi::spi1;
use apus::prelude::*;

use embedded_hal_bus::spi::ExclusiveDevice;
use stm32f0xx_hal::{pac, prelude::*};

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
    rtt_init_print!();
    rprintln!("APUS-RS example: touch_theremin");

    let mut p = pac::Peripherals::take().unwrap();

    let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);

    let gpioc = p.GPIOC.split(&mut rcc);
    let gpioe = p.GPIOE.split(&mut rcc);
    let gpiof = p.GPIOF.split(&mut rcc);

    let sys_clock_hz = rcc.clocks.sysclk().0;

    let display_pins = ApusDisplayPins {
        db0: gpioe.pe0,
        db1: gpioe.pe1,
        db2: gpioe.pe2,
        db3: gpioe.pe3,
        db4: gpioe.pe4,
        db5: gpioe.pe5,
        db6: gpioe.pe6,
        db7: gpioe.pe7,
        n_cs: gpioe.pe8,
        d_c: gpioe.pe9,
        n_wr: gpioe.pe10,
        n_rd: gpioe.pe11,
        n_reset: gpiof.pf10,
    };
    let mut display = display::init(display_pins, BusyDelay::new(sys_clock_hz));

    let (spi_bus, cs_pin) = spi1::init(
        p.SPI1, gpioe.pe12, gpioe.pe13, gpioe.pe14, gpioe.pe15, &mut rcc,
    );
    let spi_dev = ExclusiveDevice::new(spi_bus, cs_pin, BusyDelay::new(sys_clock_hz)).unwrap();
    let mut touch = Touch::new(
        spi_dev,
        touch::DEFAULT_THRESHOLD,
        DisplaySize {
            x: DISPLAY_WIDTH,
            y: DISPLAY_HEIGHT,
        },
    );

    let mut buzzer = Buzzer::new(gpioc.pc9);
    let mut delay = BusyDelay::new(sys_clock_hz);

    // calibration
    rprintln!("touch each marker as it appears");
    let calibration = touch.calibrate_from_display(&mut display);
    touch.calibrate_from_data(calibration);
    rprintln!("calibration complete - touch to play");

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
