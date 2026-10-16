#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus::bsp::leds::Leds;
use stm32f0xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    // rtt
    rtt_init_print!();
    rprintln!("APUS-RS example: leds");
    // peripherals
    let mut p = pac::Peripherals::take().unwrap();
    let cp = pac::CorePeripherals::take().unwrap();
    // clocks
    let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);
    // gpio
    let gpioc = p.GPIOC.split(&mut rcc);

    // construct LEDs
    let mut leds = Leds::new(
        gpioc.pc0, gpioc.pc1, gpioc.pc2, gpioc.pc6, gpioc.pc7, gpioc.pc8,
    );

    // delay
    let sys_clock_hz = rcc.clocks.sysclk().0;
    let mut delay = cortex_m::delay::Delay::new(cp.SYST, sys_clock_hz);

    let main_leds = leds.main_leds_mut();
    let mut previous_led = main_leds.len() - 1;

    loop {
        for i in 0..main_leds.len() {
            main_leds[previous_led].off();
            main_leds[i].on();

            delay.delay_ms(500);
            previous_led = i;
        }
    }
}
