#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use panic_probe as _;

use apus::board::{resources::Resources, RGB_PWM_FREQUENCY};
use apus::bsp::leds::Leds;
use apus::hal::pac;

#[entry]
fn main() -> ! {
    defmt::info!("APUS-RS example: leds");
    // peripherals
    let p = pac::Peripherals::take().unwrap();
    let cp = pac::CorePeripherals::take().unwrap();
    let mut res = Resources::new(p);

    // construct LEDs
    let mut leds = Leds::new(
        res.leds,
        res.peripherals.tim3,
        RGB_PWM_FREQUENCY,
        &mut res.rcc,
    );

    // delay
    let sys_clock_hz = res.rcc.clocks.sysclk().0;
    let mut delay = cortex_m::delay::Delay::new(cp.SYST, sys_clock_hz);

    let main_leds = leds.main_leds_mut();
    let mut previous_led = main_leds.len() - 1;

    loop {
        for i in 0..main_leds.len() {
            main_leds[previous_led].off();
            main_leds[i].on();

            delay.delay_ms(500);
            previous_led = i;
        }
    }
}
