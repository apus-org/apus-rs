use embedded_hal::digital::InputPin;

pub struct Button<PIN> {
    pub pin: PIN,
}

pub trait ButtonLike {
    fn pressed(&mut self) -> bool;
}

impl<PIN> Button<PIN>
where
    PIN: InputPin,
{
    pub fn new(pin: PIN) -> Self {
        Self { pin }
    }
}

impl<PIN> ButtonLike for Button<PIN>
where
    PIN: InputPin,
{
    fn pressed(&mut self) -> bool {
        let state = self.pin.is_high();
        state.unwrap_or(false)
    }
}
