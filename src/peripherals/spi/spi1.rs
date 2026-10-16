use stm32f0xx_hal::{
    gpio::{self, gpioe, Floating, Input, Output, PushPull},
    pac, rcc,
    spi::{self, Spi},
};

// Concrete SPI1 pin types
type Spi1Sck = gpioe::PE13<gpio::Alternate<gpio::AF1>>;
type Spi1Miso = gpioe::PE14<gpio::Alternate<gpio::AF1>>;
type Spi1Mosi = gpioe::PE15<gpio::Alternate<gpio::AF1>>;

pub type Spi1 = Spi<pac::SPI1, Spi1Sck, Spi1Miso, Spi1Mosi, spi::EightBit>;

use cortex_m::interrupt;
use stm32f0xx_hal::prelude::*;

/// Pins of SPI1 and the touch controller's CS, unconfigured
pub struct Spi1Pins {
    pub n_cs: gpioe::PE12<Input<Floating>>,
    pub sck: gpioe::PE13<Input<Floating>>,
    pub miso: gpioe::PE14<Input<Floating>>,
    pub mosi: gpioe::PE15<Input<Floating>>,
}

/// Initialize SPI1 with PE13/PE14/PE15 and PE12 as CS, return the bus and the CS pin
pub fn init(
    spi1_peripheral: pac::SPI1,
    pins: Spi1Pins,
    rcc: &mut rcc::Rcc,
) -> (Spi1, gpioe::PE12<Output<PushPull>>) {
    let n_cs = pins.n_cs;
    let (spi1, cs_pin) = interrupt::free(|cs| {
        let sck = pins.sck.into_alternate_af1(cs);
        let miso = pins.miso.into_alternate_af1(cs);
        let mosi = pins.mosi.into_alternate_af1(cs);

        let pins = (sck, miso, mosi);

        let mode = spi::Mode {
            polarity: spi::Polarity::IdleLow,
            phase: spi::Phase::CaptureOnFirstTransition,
        };

        // approximate prescaler=128 for 48 MHz pclk -> 375 kHz
        let speed = 375.khz();

        let spi1 = Spi::spi1(spi1_peripheral, pins, mode, speed, rcc);

        let n_cs = n_cs.into_push_pull_output(cs);

        (spi1, n_cs)
    });

    (spi1, cs_pin)
}
