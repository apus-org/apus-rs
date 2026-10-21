#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus::board::{resources::Resources, I2C1_FREQUENCY};
use apus::bsp::{delay::BusyDelay, temperature};
use apus::peripherals::i2c::i2c1;
use apus::prelude::*;

use apus::hal::pac;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("APUS-RS example: temperature");

    let p = pac::Peripherals::take().unwrap();
    let mut res = Resources::new(p);

    // MCP9803 is the only I2C bus consumer here
    let i2c = i2c1::init(res.peripherals.i2c1, res.i2c1, I2C1_FREQUENCY, &mut res.rcc);
    let mut temperature = temperature::new(i2c);

    if temperature.enable().is_err() {
        rprintln!("failed to enable temperature sensor");
    }

    let sys_clock_hz = res.rcc.clocks.sysclk().0;
    let mut delay = BusyDelay::new(sys_clock_hz);

    loop {
        match temperature.read_temperature() {
            Ok(c) => rprintln!("temp: {:.2} C", c),
            Err(_) => rprintln!("temp read error"),
        }
        delay.delay_ms(1000);
    }
}
