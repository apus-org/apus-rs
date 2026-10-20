use cortex_m::interrupt;
use stm32f0xx_hal::{
    adc,
    gpio::{gpioc, Analog, Floating, Input},
};

type MilliVolts = u16;

pub const MAX_VALUE: MilliVolts = 3300;
pub const MIN_VALUE: MilliVolts = 0;

pub struct Potentiometer {
    pin: gpioc::PC4<Analog>,
}

impl Potentiometer {
    pub fn new(pin: gpioc::PC4<Input<Floating>>) -> Self {
        let pin = interrupt::free(|cs| pin.into_analog(cs));

        Self { pin }
    }

    /// Converts the wiper voltage on `adc`, which other channels can use in between
    pub fn read(&mut self, adc: &mut adc::Adc) -> MilliVolts {
        adc.read_abs_mv(&mut self.pin)
    }
}
