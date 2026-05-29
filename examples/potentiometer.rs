#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_graphics::{pixelcolor::Rgb565, prelude::RgbColor};
use embedded_hal_compat::ForwardCompat;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus_rs::bsp::{
    display::{display_init, ApusDisplayPins},
    potentiometer::{Potentiometer, MAX_VALUE},
};
use stm32f0xx_hal::{adc::Adc, pac, prelude::*};

mod common;

const POTI_THRESHOLD_PERCENT: u32 = 5;

#[entry]
fn main() -> ! {
    // rtt
    rtt_init_print!();
    rprintln!("APUS-RS example: potentiometer");
    // peripherals
    let mut p = pac::Peripherals::take().unwrap();
    let cp = pac::CorePeripherals::take().unwrap();
    // clocks
    let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);
    let gpioc = p.GPIOC.split(&mut rcc);
    let gpioe = p.GPIOE.split(&mut rcc);
    let gpiof = p.GPIOF.split(&mut rcc);

    // delay for display initialisation
    let sys_clock_hz = rcc.clocks.sysclk().0;
    let delay_display = cortex_m::delay::Delay::new(cp.SYST, sys_clock_hz);

    // display init
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

    let mut display = display_init(display_pins, delay_display.forward());

    let adc = Adc::new(p.ADC, &mut rcc);
    let mut potentiometer = Potentiometer::new(gpioc.pc4, adc);

    let mut prev_value = 0u16;

    loop {
        let value = potentiometer.read();
        let diff: u32 = value.abs_diff(prev_value) as u32;
        if diff.saturating_mul(100) / MAX_VALUE as u32 >= POTI_THRESHOLD_PERCENT {
            // update only the delta (this example: white bar on black background)
            common::poti_draw_delta(
                &mut display,
                prev_value,
                value,
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
