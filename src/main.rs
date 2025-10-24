#![no_std]
#![no_main]

use cortex_m::asm::nop;
use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

use core::fmt::Write;
use cortex_m_semihosting::hio;

// printing with semihosting
fn print() -> Result<(), core::fmt::Error> {
    let mut stdout = hio::hstdout().map_err(|_| core::fmt::Error)?;
    let language = "Rust";
    let ranking = 1;

    write!(stdout, "{} on embedded is #{}!\n", language, ranking)?;

    Ok(())
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello World!");

    loop {
        rprintln!("Echo...");
        for _ in 0..10_000 {
            nop();
        }
    }
}
