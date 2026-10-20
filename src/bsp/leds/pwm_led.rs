use core::convert::Infallible;

use embedded_hal::{
    digital,
    pwm::{self, SetDutyCycle},
};
use stm32f0xx_hal::{pac, prelude::_embedded_hal_PwmPin, pwm::PwmChannels};

/// Active-low LED on a TIM3 channel, dimmed by its duty cycle; full duty is full brightness
pub struct PwmLed<CHANNEL> {
    channel: PwmChannels<pac::TIM3, CHANNEL>,
}

impl<CHANNEL> PwmLed<CHANNEL>
where
    PwmChannels<pac::TIM3, CHANNEL>: _embedded_hal_PwmPin<Duty = u16>,
{
    /// Takes over a running channel and switches the LED off
    pub fn new(channel: PwmChannels<pac::TIM3, CHANNEL>) -> Self {
        let mut led = Self { channel };
        led.channel.set_duty(led.period());
        led.channel.enable();
        led
    }

    /// Timer ticks per period; the output is high while the counter is below the duty value
    fn period(&self) -> u16 {
        self.channel.get_max_duty() + 1
    }
}

impl<CHANNEL> pwm::ErrorType for PwmLed<CHANNEL> {
    type Error = Infallible;
}

impl<CHANNEL> pwm::SetDutyCycle for PwmLed<CHANNEL>
where
    PwmChannels<pac::TIM3, CHANNEL>: _embedded_hal_PwmPin<Duty = u16>,
{
    fn max_duty_cycle(&self) -> u16 {
        self.period()
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        // the LED lights while the pin is low
        let period = self.period();
        self.channel.set_duty(period - duty.min(period));
        Ok(())
    }
}

impl<CHANNEL> digital::ErrorType for PwmLed<CHANNEL> {
    type Error = Infallible;
}

impl<CHANNEL> digital::OutputPin for PwmLed<CHANNEL>
where
    PwmChannels<pac::TIM3, CHANNEL>: _embedded_hal_PwmPin<Duty = u16>,
{
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_duty_cycle_fully_on()
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_duty_cycle_fully_off()
    }
}
