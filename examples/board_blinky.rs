#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use panic_probe as _;

use apus::board::Board;
use apus::prelude::*;

use apus::hal::pac;

#[entry]
fn main() -> ! {
    defmt::info!("APUS-RS example: board_blinky");

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
