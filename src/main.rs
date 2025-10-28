#![no_std]
#![no_main]

use cortex_m::asm::delay;
use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

use crate::bsp::leds::led::LedLike;
use stm32f0xx_hal::{pac, prelude::*};

mod bsp;
use bsp::leds::Leds;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello World!");

    let mut dp = pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.configure().freeze(&mut dp.FLASH);
    let gpioc = dp.GPIOC.split(&mut rcc);
    let mut leds = Leds::new(gpioc);

    for led in leds.all_mut().iter_mut() {
        led.off();
    }

    loop {
        rprintln!("Echo...");

        leds.led3.on();
        leds.led4.on();
        leds.led5.on();

        delay(5_000_000);

        leds.led3.off();
        leds.led4.off();
        leds.led5.off();

        delay(5_000_000);
    }
}
