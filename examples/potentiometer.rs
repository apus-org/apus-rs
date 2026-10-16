#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_graphics::{pixelcolor::Rgb565, prelude::RgbColor};
use embedded_hal_compat::ForwardCompat;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus::board::resources::Resources;
use apus::bsp::{
    display::{self, DISPLAY_WIDTH},
    potentiometer::{Potentiometer, MAX_VALUE},
};
use stm32f0xx_hal::{adc::Adc, pac};

mod common;

const POTI_THRESHOLD_PERCENT: u32 = 5;

#[entry]
fn main() -> ! {
    // rtt
    rtt_init_print!();
    rprintln!("APUS-RS example: potentiometer");
    // peripherals
    let p = pac::Peripherals::take().unwrap();
    let cp = pac::CorePeripherals::take().unwrap();
    let mut res = Resources::new(p);

    // delay for display initialisation
    let sys_clock_hz = res.rcc.clocks.sysclk().0;
    let delay_display = cortex_m::delay::Delay::new(cp.SYST, sys_clock_hz);

    let mut display = display::init(res.display, delay_display.forward());

    let adc = Adc::new(res.peripherals.adc, &mut res.rcc);
    let mut potentiometer = Potentiometer::new(res.potentiometer, adc);

    let mut prev_value = 0u16;

    loop {
        let value = potentiometer.read();
        let diff: u32 = value.abs_diff(prev_value) as u32;
        if diff.saturating_mul(100) / MAX_VALUE as u32 >= POTI_THRESHOLD_PERCENT {
            // update only the delta (this example: white bar on black background)
            let prev_w = (prev_value as u32 * DISPLAY_WIDTH as u32 / MAX_VALUE as u32) as u16;
            let curr_w = (value as u32 * DISPLAY_WIDTH as u32 / MAX_VALUE as u32) as u16;
            common::hbar_draw_delta(
                &mut display,
                prev_w,
                curr_w,
                Rgb565::WHITE,
                Rgb565::BLACK,
                30,
                30,
            );

            rprintln!("Potentiometer value: {}", value);

            prev_value = value;
        }
    }
}
