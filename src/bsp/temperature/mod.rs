use cortex_m::interrupt;
use embedded_hal::i2c::I2c;
use lm75::{ic, Address, Lm75};
use stm32f0xx_hal::gpio::{gpiod, Floating, Input, PullUp};

pub use lm75::Error;

/// MCP9803 temperature sensor (LM75-compatible) as configured on the APUS board
pub type ApusTemperature<I2C> = Lm75<I2C, ic::Lm75>;

/// Alert output of the temperature sensor, driven open drain
pub type TemperatureAlert = gpiod::PD4<Input<PullUp>>;

/// The temperature sensor driver and its alert output
pub struct Temperature<I2C> {
    pub sensor: ApusTemperature<I2C>,
    pub alert: TemperatureAlert,
}

impl<I2C, E> Temperature<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Construct the driver and pull up the alert output
    pub fn new(i2c: I2C, alert: gpiod::PD4<Input<Floating>>) -> Self {
        let alert = interrupt::free(|cs| alert.into_pull_up_input(cs));

        Self {
            sensor: new(i2c),
            alert,
        }
    }
}

/// Construct the temperature sensor driver at the board's I2C address (0x48).
pub fn new<I2C, E>(i2c: I2C) -> ApusTemperature<I2C>
where
    I2C: I2c<Error = E>,
{
    Lm75::new(i2c, Address::default())
}
