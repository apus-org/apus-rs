#![no_std]
#![no_main]

use core::fmt::Write;

use cortex_m_rt::entry;
use defmt_rtt as _;
use nb::block;
use panic_probe as _;

use apus::board::{defaults::UART_BAUD_RATE, resources::Resources};
use apus::peripherals::serial::usart4;

use apus::hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    defmt::info!("APUS-RS example: uart_echo");

    let p = pac::Peripherals::take().unwrap();
    let mut res = Resources::new(p);

    let mut uart = usart4::init(
        res.peripherals.usart4,
        res.usart4,
        UART_BAUD_RATE,
        &mut res.rcc,
    );

    let _ = writeln!(uart, "APUS UART echo, {} baud\r", UART_BAUD_RATE.0);

    loop {
        match block!(uart.read()) {
            Ok(byte) => {
                let _ = block!(uart.write(byte));
                // terminals send CR for Enter, answer with a full line break
                if byte == b'\r' {
                    let _ = block!(uart.write(b'\n'));
                }
            }
            Err(error) => defmt::error!("read error: {}", defmt::Debug2Format(&error)),
        }
    }
}
