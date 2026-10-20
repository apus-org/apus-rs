use cortex_m::interrupt;
use stm32f0xx_hal::gpio::{gpioa, Alternate, Floating, Input, AF0};

/// Pins of the debug port, unconfigured
pub struct SwdPins {
    pub swdio: gpioa::PA13<Input<Floating>>,
    pub swclk: gpioa::PA14<Input<Floating>>,
}

/// Pins of the debug port in their SWD function; reconfiguring them disconnects the debugger
pub struct Swd {
    pub swdio: gpioa::PA13<Alternate<AF0>>,
    pub swclk: gpioa::PA14<Alternate<AF0>>,
}

/// Type PA13 (SWDIO) and PA14 (SWCLK) as the SWD function they have since reset; the debugger
/// stays connected
pub fn init(pins: SwdPins) -> Swd {
    interrupt::free(|cs| Swd {
        swdio: pins.swdio.into_alternate_af0(cs),
        swclk: pins.swclk.into_alternate_af0(cs),
    })
}
