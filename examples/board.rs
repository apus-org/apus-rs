#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus_rs::board::Board;
use apus_rs::bsp::leds::led::LedLike;
use apus_rs::bsp::rtc::{DateTimeAccess, NaiveDate};

use embedded_hal::delay::DelayNs;
use stm32f0xx_hal::pac;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("APUS-RS example: board");

    let p = pac::Peripherals::take().unwrap();
    let mut board = Board::new(p);

    if board.temperature.enable().is_err() {
        rprintln!("failed to enable temperature sensor");
    }

    // Set a known time on each run, then start the oscillator.
    let datetime = NaiveDate::from_ymd_opt(2026, 5, 30)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    if board.rtc.set_datetime(&datetime).is_err() {
        rprintln!("failed to set datetime");
    }
    if board.rtc.enable().is_err() {
        rprintln!("failed to start rtc oscillator");
    }

    let mut led_on = false;
    loop {
        match board.temperature.read_temperature() {
            Ok(c) => rprintln!("temp: {:.2} C", c),
            Err(_) => rprintln!("temp read error"),
        }
        match board.rtc.datetime() {
            Ok(dt) => rprintln!("time: {}", dt),
            Err(_) => rprintln!("rtc read error"),
        }
        led_on = !led_on;
        if led_on {
            board.leds.led3.on();
        } else {
            board.leds.led3.off();
        }
        board.delay.delay_ms(1000);
    }
}
