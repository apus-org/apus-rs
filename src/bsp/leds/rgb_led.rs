use cortex_m::interrupt;
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_hal::pwm::SetDutyCycle;
use stm32f0xx_hal::{
    gpio::{gpioc, Floating, Input},
    pac, pwm,
    pwm::{C1, C2, C3},
    rcc::Rcc,
    time::Hertz,
};

use crate::bsp::leds::{led::LedLike, pwm_led::PwmLed};

/// Pins of the RGB LED, unconfigured
pub struct RgbPins {
    pub red: gpioc::PC6<Input<Floating>>,
    pub green: gpioc::PC7<Input<Floating>>,
    pub blue: gpioc::PC8<Input<Floating>>,
}

/// The three channels of the RGB LED, dimmable on TIM3
pub struct RgbLed {
    pub red: PwmLed<C1>,
    pub green: PwmLed<C2>,
    pub blue: PwmLed<C3>,
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
