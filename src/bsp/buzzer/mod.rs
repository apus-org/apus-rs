use cortex_m::interrupt;
use stm32f0xx_hal::{
    gpio::{gpioc, Floating, Input, Output, PushPull},
    prelude::_embedded_hal_gpio_OutputPin,
};

pub struct Buzzer {
    pin: gpioc::PC9<Output<PushPull>>,
}

impl Buzzer {
    pub fn new(buzzer: gpioc::PC9<Input<Floating>>) -> Self {
        let mut buzzer = interrupt::free(|cs| {
            let pin = buzzer.into_push_pull_output(cs);

            Self { pin }
        });

        buzzer.off();
        buzzer
    }

    pub fn on(&mut self) {
        let _ = self.pin.set_low();
    }

    pub fn off(&mut self) {
        let _ = self.pin.set_high();
    }
}
