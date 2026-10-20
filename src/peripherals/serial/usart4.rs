use cortex_m::interrupt;
use stm32f0xx_hal::{
    gpio::{gpioc, Alternate, Floating, Input, AF0},
    pac, rcc,
    serial::Serial,
    time::Bps,
};

// Concrete USART4 pin types
type Usart4Tx = gpioc::PC10<Alternate<AF0>>;
type Usart4Rx = gpioc::PC11<Alternate<AF0>>;

pub type Usart4 = Serial<pac::USART4, Usart4Tx, Usart4Rx>;

/// Pins of USART4, wired to the UART connector, unconfigured
pub struct Usart4Pins {
    pub tx: gpioc::PC10<Input<Floating>>,
    pub rx: gpioc::PC11<Input<Floating>>,
}

/// Initialize USART4 with PC10 (TX) and PC11 (RX) at `baud_rate`.
pub fn init(usart4: pac::USART4, pins: Usart4Pins, baud_rate: Bps, rcc: &mut rcc::Rcc) -> Usart4 {
    let (tx, rx) = interrupt::free(|cs| {
        let tx = pins.tx.into_alternate_af0(cs);
        let rx = pins.rx.into_alternate_af0(cs);
        (tx, rx)
    });

    Serial::usart4(usart4, (tx, rx), baud_rate, rcc)
}
