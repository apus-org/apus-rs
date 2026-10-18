use stm32f0xx_hal::{
    gpio::{gpioa, Floating, Input},
    pac,
    usb::{Peripheral, UsbBus, UsbBusType},
};
use usb_device::bus::UsbBusAllocator;

/// USB bus allocator on the board's DATA port
pub type ApusUsbBus = UsbBusAllocator<UsbBusType>;

/// Pins of the USB DATA port, unconfigured
pub struct UsbPins {
    pub dm: gpioa::PA11<Input<Floating>>,
    pub dp: gpioa::PA12<Input<Floating>>,
}

/// Initialize USB on PA11 (D-) and PA12 (D+), return the bus allocator.
///
/// The board appears on the host only once a `UsbDevice` is built from the allocator.
pub fn init(usb: pac::USB, pins: UsbPins) -> ApusUsbBus {
    UsbBus::new(Peripheral {
        usb,
        pin_dm: pins.dm,
        pin_dp: pins.dp,
    })
}
