#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus_rs::bsp::{
    buzzer::Buzzer,
};
use stm32f0xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! { 
    // rtt
    rtt_init_print!();
    rprintln!("APUS-RS example: buzzer");
    // peripherals
    let mut p = pac::Peripherals::take().unwrap();
    let cp = pac::CorePeripherals::take().unwrap();
    // clocks
    let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);
     // gpio
    let gpioc = p.GPIOC.split(&mut rcc);

    // delay
    let sys_clock_hz = rcc.clocks.sysclk().0;
    let mut delay = cortex_m::delay::Delay::new(cp.SYST, sys_clock_hz);

    // buzzer
    let mut buzzer = Buzzer::new(gpioc.pc9);

    loop {
        buzzer.on();
        delay.delay_ms(100);
        buzzer.off();
        delay.delay_ms(900);
    }
}