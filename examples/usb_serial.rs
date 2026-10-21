#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus::board::resources::Resources;
use apus::peripherals::usb;

use apus::hal::pac;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

// shared test VID/PID for CDC-ACM devices, not for products
const VID_PID: UsbVidPid = UsbVidPid(0x16c0, 0x27dd);

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("APUS-RS example: usb_serial");

    let p = pac::Peripherals::take().unwrap();
    let res = Resources::new(p);

    let usb_bus = usb::init(res.peripherals.usb, res.usb);
    let mut serial = SerialPort::new(&usb_bus);
    let mut device = UsbDeviceBuilder::new(&usb_bus, VID_PID)
        .manufacturer("HSD")
        .product("APUS serial echo")
        .serial_number("APUS-RS")
        .device_class(USB_CLASS_CDC)
        .build();

    let mut buf = [0u8; 64];
    let mut state = UsbDeviceState::Default;

    loop {
        if device.state() != state {
            state = device.state();
            rprintln!("usb state: {:?}", state);
        }

        if !device.poll(&mut [&mut serial]) {
            continue;
        }

        let count = match serial.read(&mut buf) {
            Ok(count) => count,
            Err(_) => continue,
        };

        for &byte in &buf[..count] {
            let _ = serial.write(&[byte]);
            // terminals send CR for Enter, answer with a full line break
            if byte == b'\r' {
                let _ = serial.write(b"\n");
            }
        }
    }
}
