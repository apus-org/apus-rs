#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus::board::{resources::Resources, I2C1_FREQUENCY};
use apus::bsp::{
    delay::BusyDelay,
    rtc::{self, DateTimeAccess, NaiveDate},
};
use apus::peripherals::i2c::i2c1;
use apus::prelude::*;

use stm32f0xx_hal::pac;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("APUS-RS example: rtc");

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
        rprintln!("failed to set datetime");
    }
    if rtc.enable().is_err() {
        rprintln!("failed to start oscillator");
    }

    let sys_clock_hz = res.rcc.clocks.sysclk().0;
    let mut delay = BusyDelay::new(sys_clock_hz);

    loop {
        match rtc.datetime() {
            Ok(dt) => rprintln!("{}", dt),
            Err(_) => rprintln!("rtc read error"),
        }
        delay.delay_ms(1000);
    }
}
