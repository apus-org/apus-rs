pub mod led;
pub mod pwm_led;

use cortex_m::interrupt;
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_hal::pwm::SetDutyCycle;
use led::Led;
use pwm_led::PwmLed;
use stm32f0xx_hal::{
    gpio::{gpioc, Floating, Input, Output, PushPull},
    pac, pwm,
    pwm::{C1, C2, C3},
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

/// Pins of the RGB LED, unconfigured
pub struct RgbPins {
    pub red: gpioc::PC6<Input<Floating>>,
    pub green: gpioc::PC7<Input<Floating>>,
    pub blue: gpioc::PC8<Input<Floating>>,
}

/// Represents all LEDs on the board
pub struct Leds {
    pub led3: Led<gpioc::PC0<Output<PushPull>>>,
    pub led4: Led<gpioc::PC1<Output<PushPull>>>,
    pub led5: Led<gpioc::PC2<Output<PushPull>>>,
    pub rgb: RgbLed,
}

/// The three channels of the RGB LED, dimmable on TIM3
pub struct RgbLed {
    pub red: PwmLed<C1>,
    pub green: PwmLed<C2>,
    pub blue: PwmLed<C3>,
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

impl RgbLed {
    /// Runs TIM3 at `frequency` with all channels off
    pub fn new(pins: RgbPins, tim3: pac::TIM3, frequency: Hertz, rcc: &mut Rcc) -> Self {
        let pins = interrupt::free(|cs| {
            (
                pins.red.into_alternate_af0(cs),
                pins.green.into_alternate_af0(cs),
                pins.blue.into_alternate_af0(cs),
            )
        });
        let (red, green, blue) = pwm::tim3(tim3, pins, rcc, frequency);

        Self {
            red: PwmLed::new(red),
            green: PwmLed::new(green),
            blue: PwmLed::new(blue),
        }
    }

    /// Sets all three channels, 0 is off and 255 full brightness
    pub fn set_color(&mut self, color: Rgb888) {
        let _ = self
            .red
            .set_duty_cycle_fraction(color.r().into(), u8::MAX.into());
        let _ = self
            .green
            .set_duty_cycle_fraction(color.g().into(), u8::MAX.into());
        let _ = self
            .blue
            .set_duty_cycle_fraction(color.b().into(), u8::MAX.into());
    }

    /// All channels as a mutable slice for iteration
    pub fn all_mut(&mut self) -> [&mut dyn LedLike; 3] {
        [&mut self.red, &mut self.green, &mut self.blue]
    }
}
