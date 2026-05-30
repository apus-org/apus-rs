#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus_rs::board::Board;
use apus_rs::bsp::leds::led::LedLike;

use embedded_hal::delay::DelayNs;
use stm32f0xx_hal::pac;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("APUS-RS example: board");

    let p = pac::Peripherals::take().unwrap();
    let mut board = Board::new(p);

    let mut led_on = false;
    loop {
        led_on = !led_on;
        if led_on {
            board.leds.led3.on();
        } else {
            board.leds.led3.off();
        }
        board.delay.delay_ms(500);
    }
}
