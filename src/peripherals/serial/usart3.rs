use cortex_m::interrupt;
use stm32f0xx_hal::{
    gpio::{gpiod, Alternate, Floating, Input, AF0},
    pac, rcc,
    serial::Serial,
    time::Bps,
};

// Concrete USART3 pin types
type Usart3Tx = gpiod::PD8<Alternate<AF0>>;
type Usart3Rx = gpiod::PD9<Alternate<AF0>>;

pub type Usart3 = Serial<pac::USART3, Usart3Tx, Usart3Rx>;

/// Pins of USART3, wired to the FTDI USB-UART of the VCOM port, unconfigured
pub struct Usart3Pins {
    pub tx: gpiod::PD8<Input<Floating>>,
    pub rx: gpiod::PD9<Input<Floating>>,
}

/// Initialize USART3 with PD8 (TX) and PD9 (RX) at `baud_rate`.
pub fn init(usart3: pac::USART3, pins: Usart3Pins, baud_rate: Bps, rcc: &mut rcc::Rcc) -> Usart3 {
    let (tx, rx) = interrupt::free(|cs| {
        let tx = pins.tx.into_alternate_af0(cs);
        let rx = pins.rx.into_alternate_af0(cs);
        (tx, rx)
    });

    Serial::usart3(usart3, (tx, rx), baud_rate, rcc)
}
