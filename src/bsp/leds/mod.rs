pub mod led;

use cortex_m::interrupt;
use embedded_hal::digital::v2::OutputPin;
use led::Led;
use stm32f0xx_hal::gpio::gpioc::{Parts as GpioCParts, PC0, PC1, PC2, PC6, PC7, PC8};
use stm32f0xx_hal::gpio::{Output, PushPull};

use led::LedLike;

/// Represents all LEDs on the board
pub struct Leds {
    pub led3: Led<PC0<Output<PushPull>>>,
    pub led4: Led<PC1<Output<PushPull>>>,
    pub led5: Led<PC2<Output<PushPull>>>,
    pub rgb_r: Led<PC6<Output<PushPull>>>,
    pub rgb_g: Led<PC7<Output<PushPull>>>,
    pub rgb_b: Led<PC8<Output<PushPull>>>,
}

impl Leds {
    pub fn new(gpioc: GpioCParts) -> Self {
        interrupt::free(|cs| {
            let mut led3 = gpioc.pc0.into_push_pull_output(cs);
            let mut led4 = gpioc.pc1.into_push_pull_output(cs);
            let mut led5 = gpioc.pc2.into_push_pull_output(cs);
            let mut rgb_r = gpioc.pc6.into_push_pull_output(cs);
            let mut rgb_g = gpioc.pc7.into_push_pull_output(cs);
            let mut rgb_b = gpioc.pc8.into_push_pull_output(cs);

            // Set all LEDs HIGH (OFF for active-low)
            let _ = led3.set_high();
            let _ = led4.set_high();
            let _ = led5.set_high();
            let _ = rgb_r.set_high();
            let _ = rgb_g.set_high();
            let _ = rgb_b.set_high();

            Self {
                led3: Led::new(led3),
                led4: Led::new(led4),
                led5: Led::new(led5),
                rgb_r: Led::new(rgb_r),
                rgb_g: Led::new(rgb_g),
                rgb_b: Led::new(rgb_b),
            }
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
}
