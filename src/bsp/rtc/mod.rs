use embedded_hal::i2c::I2c;
use mcp794xx::{ic, interface::I2cInterface, DateTimeAccess, Mcp794xx};

pub use mcp794xx::{Error, NaiveDate, NaiveDateTime};

/// MCP79410 real-time clock
pub struct Rtc<I2C> {
    rtc: Mcp794xx<I2cInterface<I2C>, ic::Mcp79410>,
}

impl<I2C, E> Rtc<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn new(i2c: I2C) -> Self {
        let rtc = Mcp794xx::new_mcp79410(i2c);
        Self { rtc }
    }

    /// Start the oscillator (set the clock running)
    pub fn enable(&mut self) -> Result<(), Error<E>> {
        self.rtc.enable()
    }

    pub fn set_datetime(&mut self, datetime: &NaiveDateTime) -> Result<(), Error<E>> {
        self.rtc.set_datetime(datetime)
    }

    pub fn datetime(&mut self) -> Result<NaiveDateTime, Error<E>> {
        self.rtc.datetime()
    }
}
