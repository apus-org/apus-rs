use button::Button;
use cortex_m::interrupt;
use stm32f0xx_hal::gpio::{gpioa, gpiod, Floating, Input, PullDown};

use crate::bsp::buttons::button::ButtonLike;

pub mod button;

pub struct Buttons {
    pub wake_up: Button<gpioa::PA0<Input<PullDown>>>,
    pub user_0: Button<gpiod::PD0<Input<PullDown>>>,
    pub user_1: Button<gpiod::PD1<Input<PullDown>>>,
}

impl Buttons {
    pub fn new(
        wake_up: gpioa::PA0<Input<Floating>>,
        user_0: gpiod::PD0<Input<Floating>>,
        user_1: gpiod::PD1<Input<Floating>>,
    ) -> Self {
        interrupt::free(|cs| {
            let wake_up = wake_up.into_pull_down_input(cs);
            let user_0 = user_0.into_pull_down_input(cs);
            let user_1 = user_1.into_pull_down_input(cs);

            Self {
                wake_up: Button::new(wake_up),
                user_0: Button::new(user_0),
                user_1: Button::new(user_1),
            }
        })
    }

    #[allow(dead_code)]
    pub fn all(&self) -> [&dyn ButtonLike; 3] {
        [&self.wake_up, &self.user_0, &self.user_1]
    }
}
