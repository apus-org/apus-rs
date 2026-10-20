use embedded_hal::digital::OutputPin;
use inverted_pin::InvertedPin;

/// Active-low LED; the inverted pin makes high mean on
pub type Led<PIN> = InvertedPin<PIN>;

/// Switches any output pin as an LED: on is high, off is low
pub trait LedLike {
    fn on(&mut self);
    fn off(&mut self);
}

impl<PIN> LedLike for PIN
where
    PIN: OutputPin,
{
    fn on(&mut self) {
        let _ = self.set_high();
    }

    fn off(&mut self) {
        let _ = self.set_low();
    }
}
