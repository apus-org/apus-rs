use embedded_hal::digital::OutputPin;

/// Active-low LED abstraction
pub struct Led<PIN> {
    pub pin: PIN,
}

pub trait LedLike {
    fn on(&mut self);
    fn off(&mut self);
}

impl<PIN> Led<PIN>
where
    PIN: OutputPin,
{
    pub fn new(pin: PIN) -> Self {
        Self { pin }
    }
}

impl<PIN> LedLike for Led<PIN>
where
    PIN: OutputPin,
{
    fn on(&mut self) {
        let _ = self.pin.set_low(); // low = on
    }

    fn off(&mut self) {
        let _ = self.pin.set_high(); // high = off
    }
}
