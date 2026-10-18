use stm32f0xx_hal::{
    gpio::{self, gpiob, Floating, Input},
    pac, rcc,
    spi::{self, Spi},
    time::KiloHertz,
};

// Concrete SPI2 pin types
type Spi2Sck = gpiob::PB13<gpio::Alternate<gpio::AF0>>;
type Spi2Miso = gpiob::PB14<gpio::Alternate<gpio::AF0>>;
type Spi2Mosi = gpiob::PB15<gpio::Alternate<gpio::AF0>>;

pub type Spi2 = Spi<pac::SPI2, Spi2Sck, Spi2Miso, Spi2Mosi, spi::EightBit>;

use cortex_m::interrupt;

/// Pins of SPI2 on the Arduino header (D13, D12, D11), unconfigured
pub struct Spi2Pins {
    pub sck: gpiob::PB13<Input<Floating>>,
    pub miso: gpiob::PB14<Input<Floating>>,
    pub mosi: gpiob::PB15<Input<Floating>>,
}

/// Initialize SPI2 with PB13/PB14/PB15 at `frequency`; chip selects belong to the shield
pub fn init(
    spi2_peripheral: pac::SPI2,
    pins: Spi2Pins,
    frequency: KiloHertz,
    rcc: &mut rcc::Rcc,
) -> Spi2 {
    interrupt::free(|cs| {
        let sck = pins.sck.into_alternate_af0(cs);
        let miso = pins.miso.into_alternate_af0(cs);
        let mosi = pins.mosi.into_alternate_af0(cs);

        let mode = spi::Mode {
            polarity: spi::Polarity::IdleLow,
            phase: spi::Phase::CaptureOnFirstTransition,
        };

        Spi::spi2(spi2_peripheral, (sck, miso, mosi), mode, frequency, rcc)
    })
}
