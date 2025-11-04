pub mod led;

use cortex_m::interrupt;
use led::Led;
use stm32f0xx_hal::gpio::{gpioc, Floating, Input, Output, PushPull};

use led::LedLike;

/// Represents all LEDs on the board
pub struct Leds {
    pub led3: Led<gpioc::PC0<Output<PushPull>>>,
    pub led4: Led<gpioc::PC1<Output<PushPull>>>,
    pub led5: Led<gpioc::PC2<Output<PushPull>>>,
    pub rgb_r: Led<gpioc::PC6<Output<PushPull>>>,
    pub rgb_g: Led<gpioc::PC7<Output<PushPull>>>,
    pub rgb_b: Led<gpioc::PC8<Output<PushPull>>>,
}

impl Leds {
    pub fn new(
        led3: gpioc::PC0<Input<Floating>>,
        led4: gpioc::PC1<Input<Floating>>,
        led5: gpioc::PC2<Input<Floating>>,
        rgb_r: gpioc::PC6<Input<Floating>>,
        rgb_g: gpioc::PC7<Input<Floating>>,
        rgb_b: gpioc::PC8<Input<Floating>>,
    ) -> Self {
        interrupt::free(|cs| {
            let led3 = led3.into_push_pull_output(cs);
            let led4 = led4.into_push_pull_output(cs);
            let led5 = led5.into_push_pull_output(cs);
            let rgb_r = rgb_r.into_push_pull_output(cs);
            let rgb_g = rgb_g.into_push_pull_output(cs);
            let rgb_b = rgb_b.into_push_pull_output(cs);

            let mut leds = Self {
                led3: Led::new(led3),
                led4: Led::new(led4),
                led5: Led::new(led5),
                rgb_r: Led::new(rgb_r),
                rgb_g: Led::new(rgb_g),
                rgb_b: Led::new(rgb_b),
            };

            // initialise all leds as off
            leds.all_mut().iter_mut().for_each(|led| led.off());

            leds
        })
    }

    /// All LEDs as a mutable slice for iteration
    pub fn all_mut(&mut self) -> [&mut dyn LedLike; 6] {
        [
            &mut self.led3,
            &mut self.led4,
            &mut self.led5,
            &mut self.rgb_r,
            &mut self.rgb_g,
            &mut self.rgb_b,
        ]
    }

    pub fn main_leds_mut(&mut self) -> [&mut dyn LedLike; 3] {
        [&mut self.led3, &mut self.led4, &mut self.led5]
    }
}
