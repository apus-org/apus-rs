#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use panic_probe as _;

use apus::board::Board;
use apus::hal::pac;
use apus::prelude::*;

#[entry]
fn main() -> ! {
    defmt::info!("{{project-name}}");

    let p = pac::Peripherals::take().unwrap();
    let mut board = Board::new(p);

    loop {
        board.leds.led3.on();
        board.delay.delay_ms(500);
        board.leds.led3.off();
        board.delay.delay_ms(500);
    }
}
