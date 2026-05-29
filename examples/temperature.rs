#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus_rs::bsp::{delay::BusyDelay, temperature::temperature_init};
use apus_rs::peripherals::i2c::i2c1::init_i2c1;

use embedded_hal::delay::DelayNs;
use stm32f0xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("APUS-RS example: temperature");

    let mut p = pac::Peripherals::take().unwrap();
    let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);
    let gpiob = p.GPIOB.split(&mut rcc);

    // MCP9803 is the only I2C bus consumer here
    let i2c = init_i2c1(p.I2C1, gpiob.pb6, gpiob.pb7, &mut rcc);
    let mut temperature = temperature_init(i2c);

    if temperature.enable().is_err() {
        rprintln!("failed to enable temperature sensor");
    }

    let sys_clock_hz = rcc.clocks.sysclk().0;
    let mut delay = BusyDelay::new(sys_clock_hz);

    loop {
        match temperature.read_temperature() {
            Ok(c) => rprintln!("temp: {:.2} C", c),
            Err(_) => rprintln!("temp read error"),
        }
        delay.delay_ms(1000);
    }
}
