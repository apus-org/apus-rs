#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use panic_probe as _;

use apus::board::{defaults::I2C1_FREQUENCY, resources::Resources};
use apus::bsp::{delay::BusyDelay, temperature};
use apus::peripherals::i2c::i2c1;
use apus::prelude::*;

use apus::hal::pac;

#[entry]
fn main() -> ! {
    defmt::info!("APUS-RS example: temperature");

    let p = pac::Peripherals::take().unwrap();
    let mut res = Resources::new(p);

    // MCP9803 is the only I2C bus consumer here
    let i2c = i2c1::init(res.peripherals.i2c1, res.i2c1, I2C1_FREQUENCY, &mut res.rcc);
    let mut temperature = temperature::new(i2c);

    if temperature.enable().is_err() {
        defmt::error!("failed to enable temperature sensor");
    }

    let sys_clock_hz = res.rcc.clocks.sysclk().0;
    let mut delay = BusyDelay::new(sys_clock_hz);

    loop {
        match temperature.read_temperature() {
            Ok(c) => defmt::info!("temp: {} C", c),
            Err(_) => defmt::error!("temp read error"),
        }
        delay.delay_ms(1000);
    }
}
