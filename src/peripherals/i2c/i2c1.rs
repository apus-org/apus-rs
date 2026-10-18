use stm32f0xx_hal::{
    gpio::{gpiob, Alternate, Floating, Input, AF1},
    i2c::I2c,
    pac, rcc,
    time::KiloHertz,
};

use cortex_m::interrupt;

// Concrete I2C1 pin types and type alias
type I2c1Scl = gpiob::PB6<Alternate<AF1>>;
type I2c1Sda = gpiob::PB7<Alternate<AF1>>;

pub type I2c1 = I2c<pac::I2C1, I2c1Scl, I2c1Sda>;

/// Pins of I2C1, unconfigured
pub struct I2c1Pins {
    pub scl: gpiob::PB6<Input<Floating>>,
    pub sda: gpiob::PB7<Input<Floating>>,
}

/// Initialize I2C1 with PB6 (SCL) and PB7 (SDA) at `frequency`.
pub fn init(i2c1: pac::I2C1, pins: I2c1Pins, frequency: KiloHertz, rcc: &mut rcc::Rcc) -> I2c1 {
    let (scl, sda) = interrupt::free(|cs| {
        let scl = pins.scl.into_alternate_af1(cs);
        let sda = pins.sda.into_alternate_af1(cs);
        (scl, sda)
    });

    I2c::i2c1(i2c1, (scl, sda), frequency, rcc)
}
