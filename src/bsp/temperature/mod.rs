use embedded_hal::i2c::I2c;
use lm75::{Address, Lm75};

pub use lm75::Error;

/// MCP9803 temperature sensor (LM75-compatible)
pub struct Temperature<I2C> {
    sensor: Lm75<I2C, lm75::ic::Lm75>,
}

impl<I2C, E> Temperature<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn new(i2c: I2C) -> Self {
        let sensor = Lm75::new(i2c, Address::default());
        Self { sensor }
    }

    pub fn enable(&mut self) -> Result<(), Error<E>> {
        self.sensor.enable()
    }

    /// Read temperature in degrees Celsius
    pub fn read_celsius(&mut self) -> Result<f32, Error<E>> {
        self.sensor.read_temperature()
    }
}
