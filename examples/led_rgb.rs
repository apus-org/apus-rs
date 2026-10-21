#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use embedded_hal::pwm::SetDutyCycle;
use panic_probe as _;

use apus::board::{defaults::RGB_PWM_FREQUENCY, resources::Resources};
use apus::bsp::{
    delay::BusyDelay,
    leds::Leds,
    potentiometer::{self, Potentiometer},
};
use apus::prelude::*;

use apus::hal::{adc::Adc, pac};

// red, green and blue, evenly spaced on the colour wheel
const CHANNELS: u32 = 3;
// between two neighbouring channels, one rises and then the other falls
const SEGMENTS: u32 = 2 * CHANNELS;
const STEPS_PER_SEGMENT: u32 = 1000;
const POSITIONS: u32 = SEGMENTS * STEPS_PER_SEGMENT;
const STEP_MS: u32 = 1;

// a level times the largest duty cycle has to fit in u32
const _: () = assert!(STEPS_PER_SEGMENT <= u32::MAX / u16::MAX as u32);

/// Level of the channel whose colour sits at `centre`, up to STEPS_PER_SEGMENT: full up to one
/// segment away, then fading out over the next segment
fn channel(position: u32, centre: u32) -> u32 {
    let distance = position.abs_diff(centre);
    let distance = distance.min(POSITIONS - distance);
    (2 * STEPS_PER_SEGMENT)
        .saturating_sub(distance)
        .min(STEPS_PER_SEGMENT)
}

/// Levels of red, green and blue at `position`
fn wheel(position: u32) -> [u32; CHANNELS as usize] {
    core::array::from_fn(|index| channel(position, index as u32 * POSITIONS / CHANNELS))
}

/// Sets `level` (up to STEPS_PER_SEGMENT) at the timer's full resolution, scaled by
/// `brightness` (up to MAX_VALUE)
fn set_level(led: &mut impl SetDutyCycle, level: u32, brightness: u32) {
    let full_brightness = u32::from(led.max_duty_cycle()) * level / STEPS_PER_SEGMENT;
    let duty = full_brightness * brightness / u32::from(potentiometer::MAX_VALUE);
    let _ = led.set_duty_cycle(duty as u16);
}

#[entry]
fn main() -> ! {
    defmt::info!("APUS-RS example: led_rgb (turn the potentiometer for brightness)");

    let p = pac::Peripherals::take().unwrap();
    let mut res = Resources::new(p);

    let sys_clock_hz = res.rcc.clocks.sysclk().0;
    let mut delay = BusyDelay::new(sys_clock_hz);

    let mut leds = Leds::new(
        res.leds,
        res.peripherals.tim3,
        RGB_PWM_FREQUENCY,
        &mut res.rcc,
    );
    let rgb = &mut leds.rgb;

    let mut adc = Adc::new(res.peripherals.adc, &mut res.rcc);
    let mut potentiometer = Potentiometer::new(res.potentiometer);

    loop {
        for position in 0..POSITIONS {
            // squared, because the eye sees small duty cycles as much brighter than they are
            let knob = u32::from(potentiometer.read(&mut adc));
            let brightness = knob * knob / u32::from(potentiometer::MAX_VALUE);

            let [red, green, blue] = wheel(position);
            set_level(&mut rgb.red, red, brightness);
            set_level(&mut rgb.green, green, brightness);
            set_level(&mut rgb.blue, blue, brightness);

            delay.delay_ms(STEP_MS);
        }
    }
}
