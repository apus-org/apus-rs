use embedded_hal::digital::v2::InputPin;

pub struct Button<PIN> {
    pub pin: PIN,
}

pub trait ButtonLike {
    fn pressed(&self) -> bool;
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
    fn pressed(&self) -> bool {
        let state = self.pin.is_high();
        match state {
            Ok(pressed) => return pressed,
            Err(_) => return false,
        }
    }
}
