use cortex_m::interrupt;
use stm32f0xx_hal::{
    gpio::{gpioa, Alternate, Floating, Input, AF1},
    pac, rcc,
    serial::Serial,
    time::Bps,
};

// Concrete USART1 pin types
type Usart1Tx = gpioa::PA9<Alternate<AF1>>;
type Usart1Rx = gpioa::PA10<Alternate<AF1>>;

pub type Usart1 = Serial<pac::USART1, Usart1Tx, Usart1Rx>;

/// Pins of USART1 on the Arduino header (D1, D0), unconfigured
pub struct Usart1Pins {
    pub tx: gpioa::PA9<Input<Floating>>,
    pub rx: gpioa::PA10<Input<Floating>>,
}

/// Initialize USART1 with PA9 (TX) and PA10 (RX) at `baud_rate`.
pub fn init(usart1: pac::USART1, pins: Usart1Pins, baud_rate: Bps, rcc: &mut rcc::Rcc) -> Usart1 {
    let (tx, rx) = interrupt::free(|cs| {
        let tx = pins.tx.into_alternate_af1(cs);
        let rx = pins.rx.into_alternate_af1(cs);
        (tx, rx)
    });

    Serial::usart1(usart1, (tx, rx), baud_rate, rcc)
}
