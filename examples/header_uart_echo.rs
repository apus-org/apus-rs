#![no_std]
#![no_main]

use core::fmt::Write;

use cortex_m_rt::entry;
use nb::block;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus::board::resources::Resources;
use apus::peripherals::serial::usart1::{self, Usart1Pins};

use apus::hal::{pac, prelude::*, time::Bps};

const BAUD_RATE: Bps = Bps(115_200);

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("APUS-RS example: header_uart_echo");

    let p = pac::Peripherals::take().unwrap();
    let mut res = Resources::new(p);

    let header = res.header;
    let pins = Usart1Pins {
        tx: header.d1,
        rx: header.d0,
    };
    let mut uart = usart1::init(res.peripherals.free.usart1, pins, BAUD_RATE, &mut res.rcc);

    let _ = writeln!(uart, "APUS header UART echo, {} baud\r", BAUD_RATE.0);

    loop {
        match block!(uart.read()) {
            Ok(byte) => {
                let _ = block!(uart.write(byte));
                // terminals send CR for Enter, answer with a full line break
                if byte == b'\r' {
                    let _ = block!(uart.write(b'\n'));
                }
            }
            Err(error) => rprintln!("read error: {:?}", error),
        }
    }
}
