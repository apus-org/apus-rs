use stm32f0xx_hal::{
    gpio::{self, gpioe, Floating, Input, Output, PushPull},
    spi::{self, Spi},
    rcc, pac,
};

use shared_bus;

// Concrete SPI1 pin types and shared-bus type alias to avoid long types
type Spi1Sck = gpioe::PE13<gpio::Alternate<gpio::AF1>>;
type Spi1Miso = gpioe::PE14<gpio::Alternate<gpio::AF1>>;
type Spi1Mosi = gpioe::PE15<gpio::Alternate<gpio::AF1>>;

pub type Spi1 = Spi<pac::SPI1, Spi1Sck, Spi1Miso, Spi1Mosi, spi::EightBit>;

pub type Spi1Shared = shared_bus::BusManagerSimple<Spi1>;

use cortex_m::interrupt;
use stm32f0xx_hal::prelude::*;

/// Initialize SPI1 with PE13/PE14/PE15 and PE12 as CS, return a shared-bus manager and the CS pin
pub fn init_spi1(
    spi1_peripheral: pac::SPI1,
    n_cs: gpioe::PE12<Input<Floating>>,
    sck: gpioe::PE13<Input<Floating>>,
    miso: gpioe::PE14<Input<Floating>>,
    mosi: gpioe::PE15<Input<Floating>>,
    rcc: &mut rcc::Rcc,
) -> (Spi1Shared, gpioe::PE12<Output<PushPull>>) {
    let (spi1, cs_pin) = interrupt::free(|cs| {
        let sck = sck.into_alternate_af1(cs);
        let miso = miso.into_alternate_af1(cs);
        let mosi = mosi.into_alternate_af1(cs);

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

    let manager = shared_bus::BusManagerSimple::new(spi1);

    (manager, cs_pin)
}