use embedded_hal::i2c::I2c;
use lm75::{ic, Address, Lm75};

pub use lm75::Error;

/// MCP9803 temperature sensor (LM75-compatible) as configured on the APUS board
pub type ApusTemperature<I2C> = Lm75<I2C, ic::Lm75>;

/// Construct the temperature sensor driver at the board's I2C address (0x48).
pub fn temperature_init<I2C, E>(i2c: I2C) -> ApusTemperature<I2C>
where
    I2C: I2c<Error = E>,
{
    Lm75::new(i2c, Address::default())
}
