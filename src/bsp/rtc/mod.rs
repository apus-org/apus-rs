use embedded_hal::i2c::I2c;
use mcp794xx::{ic, interface::I2cInterface, Mcp794xx};

pub use mcp794xx::{DateTimeAccess, Error, NaiveDate, NaiveDateTime};

/// MCP79410 real-time clock as configured on the APUS board
pub type ApusRtc<I2C> = Mcp794xx<I2cInterface<I2C>, ic::Mcp79410>;

/// Construct the MCP79410 RTC driver.
pub fn new<I2C, E>(i2c: I2C) -> ApusRtc<I2C>
where
    I2C: I2c<Error = E>,
{
    Mcp794xx::new_mcp79410(i2c)
}
