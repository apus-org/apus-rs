#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus::bsp::buttons::Buttons;
use apus::bsp::buzzer::Buzzer;
use apus::bsp::delay::BusyDelay;
use apus::prelude::*;
use stm32f0xx_hal::{pac, prelude::*};

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

    let mut p = pac::Peripherals::take().unwrap();

    let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);

    let gpioa = p.GPIOA.split(&mut rcc);
    let gpioc = p.GPIOC.split(&mut rcc);
    let gpiod = p.GPIOD.split(&mut rcc);

    let sys_clock_hz = rcc.clocks.sysclk().0;
    let mut delay = BusyDelay::new(sys_clock_hz);

    let mut buzzer = Buzzer::new(gpioc.pc9);
    let mut buttons = Buttons::new(gpioa.pa0, gpiod.pd0, gpiod.pd1);

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
