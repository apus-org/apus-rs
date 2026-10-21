pub mod led;
pub mod pwm_led;
pub mod rgb_led;

use cortex_m::interrupt;
use led::Led;
use rgb_led::{RgbLed, RgbPins};
use stm32f0xx_hal::{
    gpio::{gpioc, Floating, Input, Output, PushPull},
    pac,
    rcc::Rcc,
    time::Hertz,
};

use led::LedLike;

/// Pins of all LEDs, unconfigured
pub struct LedPins {
    pub led3: gpioc::PC0<Input<Floating>>,
    pub led4: gpioc::PC1<Input<Floating>>,
    pub led5: gpioc::PC2<Input<Floating>>,
    pub rgb: RgbPins,
}

/// Represents all LEDs on the board
pub struct Leds {
    pub led3: Led<gpioc::PC0<Output<PushPull>>>,
    pub led4: Led<gpioc::PC1<Output<PushPull>>>,
    pub led5: Led<gpioc::PC2<Output<PushPull>>>,
    pub rgb: RgbLed,
}

impl Leds {
    /// Takes TIM3 for the RGB LED, which runs at `rgb_frequency`
    pub fn new(pins: LedPins, tim3: pac::TIM3, rgb_frequency: Hertz, rcc: &mut Rcc) -> Self {
        let rgb = RgbLed::new(pins.rgb, tim3, rgb_frequency, rcc);

        let mut leds = interrupt::free(|cs| {
            let led3 = pins.led3.into_push_pull_output(cs);
            let led4 = pins.led4.into_push_pull_output(cs);
            let led5 = pins.led5.into_push_pull_output(cs);

            Self {
                led3: Led::new(led3),
                led4: Led::new(led4),
                led5: Led::new(led5),
                rgb,
            }
        });

        // initialise all leds as off
        leds.all_mut().iter_mut().for_each(|led| led.off());

        leds
    }

    /// All LEDs as a mutable slice for iteration
    pub fn all_mut(&mut self) -> [&mut dyn LedLike; 6] {
        [
            &mut self.led3,
            &mut self.led4,
            &mut self.led5,
            &mut self.rgb.red,
            &mut self.rgb.green,
            &mut self.rgb.blue,
        ]
    }

    #[allow(dead_code)]
    pub fn main_leds_mut(&mut self) -> [&mut dyn LedLike; 3] {
        [&mut self.led3, &mut self.led4, &mut self.led5]
    }
}
