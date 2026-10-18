use cortex_m::interrupt;
use stm32f0xx_hal::{
    can::{bxcan, CanInstance},
    gpio::{gpiob, gpioc, Alternate, Floating, Input, Output, PushPull, AF4},
    pac,
    prelude::*,
    rcc,
};

// Concrete CAN pin types
type CanRx = gpiob::PB8<Alternate<AF4>>;
type CanTx = gpiob::PB9<Alternate<AF4>>;

pub type Can = CanInstance<CanTx, CanRx>;

/// bxCAN driver on the board's CAN pins
pub type ApusCan = bxcan::Can<Can>;

/// Mode pin (Rs) of the SN65HVD230 transceiver, configured
pub type CanMode = gpioc::PC13<Output<PushPull>>;

/// Pins of CAN and the transceiver's mode pin, unconfigured
pub struct CanPins {
    pub rx: gpiob::PB8<Input<Floating>>,
    pub tx: gpiob::PB9<Input<Floating>>,
    pub mode: gpioc::PC13<Input<Floating>>,
}

/// SN65HVD230 CAN transceiver
pub struct Transceiver {
    mode: CanMode,
}

impl Transceiver {
    /// Takes the mode pin and makes the transceiver active
    pub fn new(mode: CanMode) -> Self {
        let mut transceiver = Self { mode };
        transceiver.active();
        transceiver
    }

    /// Drives and receives the bus (high-speed mode)
    pub fn active(&mut self) {
        let _ = self.mode.set_low(); // low = active
    }

    /// Only listens for bus activity, with low power consumption
    pub fn standby(&mut self) {
        let _ = self.mode.set_high(); // high = standby
    }
}

/// CAN controller and transceiver of the board
pub struct CanBus {
    pub controller: ApusCan,
    pub transceiver: Transceiver,
}

impl CanBus {
    /// Takes the peripheral and the mode pin from [`init`]. The controller is left disabled:
    /// bit timing, mode and filters are up to the application.
    pub fn new(can: Can, mode: CanMode) -> Self {
        Self {
            controller: bxcan::Can::builder(can).leave_disabled(),
            transceiver: Transceiver::new(mode),
        }
    }
}

/// Initialize CAN on PB8 (RX) and PB9 (TX) with the transceiver active, return the
/// peripheral and the mode pin.
///
/// Bit timing, filters and loopback or silent mode are set with [`bxcan::Can::builder`].
pub fn init(can: pac::CAN, pins: CanPins, rcc: &mut rcc::Rcc) -> (Can, CanMode) {
    let (rx, tx, mut mode) = interrupt::free(|cs| {
        (
            pins.rx.into_alternate_af4(cs),
            pins.tx.into_alternate_af4(cs),
            pins.mode.into_push_pull_output(cs),
        )
    });

    // high-speed mode (negative logic)
    let _ = mode.set_low();

    (CanInstance::new(can, tx, rx, rcc), mode)
}
