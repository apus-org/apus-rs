use cortex_m::interrupt;
use stm32f0xx_hal::{
    gpio::{gpiod, Floating, Input, Output, PushPull},
    prelude::_embedded_hal_gpio_OutputPin,
};

use crate::peripherals::serial::usart3::Usart3;

/// Ready to receive, to the FTDI's CTS# (FTDI_CTS net); low lets the PC send
pub type VcomRts = gpiod::PD11<Output<PushPull>>;

/// The PC's ready to receive, from the FTDI's RTS# (FTDI_RTS net); low means ready
pub type VcomCts = gpiod::PD12<Input<Floating>>;

/// USART3 to the FTDI USB-UART and its flow-control lines, driven in software
pub struct Vcom {
    pub serial: Usart3,
    pub rts: VcomRts,
    pub cts: VcomCts,
}

impl Vcom {
    pub fn new(
        serial: Usart3,
        rts: gpiod::PD11<Input<Floating>>,
        cts: gpiod::PD12<Input<Floating>>,
    ) -> Self {
        let (mut rts, cts) =
            interrupt::free(|cs| (rts.into_push_pull_output(cs), cts.into_floating_input(cs)));
        let _ = rts.set_low();

        Self { serial, rts, cts }
    }
}
