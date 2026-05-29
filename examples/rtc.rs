#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus_rs::bsp::{
    delay::BusyDelay,
    rtc::{self, DateTimeAccess, NaiveDate},
};
use apus_rs::peripherals::i2c::i2c1;

use embedded_hal::delay::DelayNs;
use stm32f0xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("APUS-RS example: rtc");

    let mut p = pac::Peripherals::take().unwrap();
    let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);
    let gpiob = p.GPIOB.split(&mut rcc);

    let i2c = i2c1::init(p.I2C1, gpiob.pb6, gpiob.pb7, &mut rcc);
    let mut rtc = rtc::new(i2c);

    // Reset the clock to a known time on every run, then start it.
    // A real app would set the time once and rely on the battery-backed oscillator.
    let datetime = NaiveDate::from_ymd_opt(2026, 5, 30)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    if rtc.set_datetime(&datetime).is_err() {
        rprintln!("failed to set datetime");
    }
    if rtc.enable().is_err() {
        rprintln!("failed to start oscillator");
    }

    let sys_clock_hz = rcc.clocks.sysclk().0;
    let mut delay = BusyDelay::new(sys_clock_hz);

    loop {
        match rtc.datetime() {
            Ok(dt) => rprintln!("{}", dt),
            Err(_) => rprintln!("rtc read error"),
        }
        delay.delay_ms(1000);
    }
}
