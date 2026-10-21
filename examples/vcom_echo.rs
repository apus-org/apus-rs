#![no_std]
#![no_main]

use core::fmt::Write;

use cortex_m_rt::entry;
use nb::block;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus::board::{resources::Resources, VCOM_BAUD_RATE};
use apus::peripherals::serial::usart3;

use apus::hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("APUS-RS example: vcom_echo");

    let p = pac::Peripherals::take().unwrap();
    let mut res = Resources::new(p);

    let mut vcom = usart3::init(
        res.peripherals.usart3,
        res.usart3,
        VCOM_BAUD_RATE,
        &mut res.rcc,
    );

    let _ = writeln!(vcom, "APUS VCOM echo, {} baud\r", VCOM_BAUD_RATE.0);

    loop {
        match block!(vcom.read()) {
            Ok(byte) => {
                let _ = block!(vcom.write(byte));
                // terminals send CR for Enter, answer with a full line break
                if byte == b'\r' {
                    let _ = block!(vcom.write(b'\n'));
                }
            }
            Err(error) => rprintln!("read error: {:?}", error),
        }
    }
}
