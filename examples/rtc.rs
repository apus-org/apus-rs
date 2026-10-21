#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use panic_probe as _;

use apus::board::{defaults::I2C1_FREQUENCY, resources::Resources};
use apus::bsp::{
    delay::BusyDelay,
    rtc::{self, DateTimeAccess, NaiveDate},
};
use apus::peripherals::i2c::i2c1;
use apus::prelude::*;

use apus::hal::pac;

#[entry]
fn main() -> ! {
    defmt::info!("APUS-RS example: rtc");

    let p = pac::Peripherals::take().unwrap();
    let mut res = Resources::new(p);

    let i2c = i2c1::init(res.peripherals.i2c1, res.i2c1, I2C1_FREQUENCY, &mut res.rcc);
    let mut rtc = rtc::new(i2c);

    // Reset the clock to a known time on every run, then start it.
    // Alternatively, set the time once and rely on the battery-backed oscillator.
    let datetime = NaiveDate::from_ymd_opt(2002, 6, 10)
        .unwrap()
        .and_hms_opt(9, 0, 0)
        .unwrap();
    if rtc.set_datetime(&datetime).is_err() {
        defmt::error!("failed to set datetime");
    }
    if rtc.enable().is_err() {
        defmt::error!("failed to start oscillator");
    }

    let sys_clock_hz = res.rcc.clocks.sysclk().0;
    let mut delay = BusyDelay::new(sys_clock_hz);

    loop {
        match rtc.datetime() {
            Ok(dt) => defmt::info!("{}", defmt::Display2Format(&dt)),
            Err(_) => defmt::error!("rtc read error"),
        }
        delay.delay_ms(1000);
    }
}
