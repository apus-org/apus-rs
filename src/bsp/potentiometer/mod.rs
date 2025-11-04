use cortex_m::interrupt;
use stm32f0xx_hal::{
    adc,
    gpio::{Analog, Floating, Input, gpioc},
};

type MilliVolts = u16;

pub struct Potentiometer {
    pin : gpioc::PC4<Analog>,
    adc : adc::Adc,
}

impl Potentiometer {
    pub fn new(pin: gpioc::PC4<Input<Floating>>, adc: adc::Adc) -> Self {
        let pin = interrupt::free(|cs| {
            pin.into_analog(cs)
        });
        
        Self { pin, adc }
    }

    pub fn read(&mut self) -> MilliVolts {
        self.adc.read_abs_mv(&mut self.pin)
    }
}