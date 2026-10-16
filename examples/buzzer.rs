#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus::board::resources::Resources;
use apus::bsp::buttons::Buttons;
use apus::bsp::buzzer::Buzzer;
use apus::bsp::delay::BusyDelay;
use apus::prelude::*;
use stm32f0xx_hal::pac;

mod common;

// c-major triad: one note per button
const NOTE_C5: u32 = 523;
const NOTE_E5: u32 = 659;
const NOTE_G5: u32 = 784;

const TONE_LENGTH_MS: u32 = 20;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("APUS-RS example: buzzer (hold a button to play a note)");

    let p = pac::Peripherals::take().unwrap();

    let res = Resources::new(p);

    let sys_clock_hz = res.rcc.clocks.sysclk().0;
    let mut delay = BusyDelay::new(sys_clock_hz);

    let mut buzzer = Buzzer::new(res.buzzer);
    let mut buttons = Buttons::new(res.buttons);

    loop {
        let note = if buttons.wake_up.pressed() {
            Some(NOTE_C5)
        } else if buttons.user_0.pressed() {
            Some(NOTE_E5)
        } else if buttons.user_1.pressed() {
            Some(NOTE_G5)
        } else {
            None
        };

        match note {
            Some(freq) => common::play_tone(&mut buzzer, &mut delay, freq, TONE_LENGTH_MS),
            None => {
                buzzer.off();
                delay.delay_ms(TONE_LENGTH_MS);
            }
        }
    }
}
