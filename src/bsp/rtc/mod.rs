use cortex_m::interrupt;
use embedded_hal::i2c::I2c;
use mcp794xx::{ic, interface::I2cInterface, Mcp794xx};
use stm32f0xx_hal::gpio::{gpiod, Floating, Input, PullUp};

pub use mcp794xx::{DateTimeAccess, Error, NaiveDate, NaiveDateTime};

/// MCP79410 real-time clock as configured on the APUS board
pub type ApusRtc<I2C> = Mcp794xx<I2cInterface<I2C>, ic::Mcp79410>;

/// Multi-function pin of the RTC (alarm, square wave or output), driven open drain
pub type RtcMfp = gpiod::PD3<Input<PullUp>>;

/// The RTC driver and its multi-function pin
pub struct Rtc<I2C> {
    pub clock: ApusRtc<I2C>,
    pub mfp: RtcMfp,
}

impl<I2C, E> Rtc<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Construct the driver and pull up the multi-function pin
    pub fn new(i2c: I2C, mfp: gpiod::PD3<Input<Floating>>) -> Self {
        let mfp = interrupt::free(|cs| mfp.into_pull_up_input(cs));

        Self {
            clock: new(i2c),
            mfp,
        }
    }
}

/// Construct the MCP79410 RTC driver.
pub fn new<I2C, E>(i2c: I2C) -> ApusRtc<I2C>
where
    I2C: I2c<Error = E>,
{
    Mcp794xx::new_mcp79410(i2c)
}
