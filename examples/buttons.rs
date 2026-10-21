#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use panic_probe as _;

use apus::board::{defaults::RGB_PWM_FREQUENCY, resources::Resources};
use apus::bsp::{buttons::Buttons, leds::Leds};
use apus::hal::pac;
use apus::prelude::*;

#[entry]
fn main() -> ! {
    defmt::info!("APUS-RS example: buttons");
    // peripherals
    let p = pac::Peripherals::take().unwrap();
    let cp = pac::CorePeripherals::take().unwrap();
    let mut res = Resources::new(p);

    // construct LEDs and Buttons
    let mut leds = Leds::new(
        res.leds,
        res.peripherals.tim3,
        RGB_PWM_FREQUENCY,
        &mut res.rcc,
    );
    let mut buttons = Buttons::new(res.buttons);

    // delay for debouncing
    let sys_clock_hz = res.rcc.clocks.sysclk().0;
    let mut delay = cortex_m::delay::Delay::new(cp.SYST, sys_clock_hz);

    loop {
        // map buttons to main leds
        if buttons.wake_up.pressed() {
            leds.led3.on();
        } else {
            leds.led3.off();
        }

        if buttons.user_0.pressed() {
            leds.led4.on();
        } else {
            leds.led4.off();
        }

        if buttons.user_1.pressed() {
            leds.led5.on();
        } else {
            leds.led5.off();
        }

        delay.delay_ms(10);
    }
}
