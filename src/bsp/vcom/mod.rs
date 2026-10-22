use cortex_m::interrupt;
use stm32f0xx_hal::{
    gpio::{gpiod, Floating, Input, Output, PullUp, PushPull},
    prelude::_embedded_hal_gpio_OutputPin,
};

use crate::peripherals::serial::usart3::Usart3;

/// Ready to receive, to the FTDI's CTS# (FTDI_CTS net); low lets the PC send
pub type VcomRts = gpiod::PD11<Output<PushPull>>;

/// The PC's ready to receive, from the FTDI's RTS# (FTDI_RTS net); low means ready, and the pull-up
/// reads not ready while the VCOM cable is unplugged
pub type VcomCts = gpiod::PD12<Input<PullUp>>;

/// Flow-control lines of the VCOM port, unconfigured
pub struct VcomFlowControlPins {
    /// PD11 · ready to receive, to the FTDI's CTS# (FTDI_CTS net)
    pub rts: gpiod::PD11<Input<Floating>>,
    /// PD12 · the PC's ready to receive, from the FTDI's RTS# (FTDI_RTS net)
    pub cts: gpiod::PD12<Input<Floating>>,
}

/// USART3 to the FTDI USB-UART and its flow-control lines, driven in software
pub struct Vcom {
    pub serial: Usart3,
    pub rts: VcomRts,
    pub cts: VcomCts,
}

impl Vcom {
    pub fn new(serial: Usart3, pins: VcomFlowControlPins) -> Self {
        let (mut rts, cts) = interrupt::free(|cs| {
            (
                pins.rts.into_push_pull_output(cs),
                pins.cts.into_pull_up_input(cs),
            )
        });
        let _ = rts.set_low();

        Self { serial, rts, cts }
    }
}
