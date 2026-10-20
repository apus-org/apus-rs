#![no_std]
#![no_main]

use core::cell::RefCell;

use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::{Alignment, Text},
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus::board::Board;
use apus::bsp::display::ApusDisplay;
use apus::peripherals::usb::ApusUsbBus;

use stm32f0xx_hal::pac::{self, interrupt, Interrupt};
use stm32f0xx_hal::usb::UsbBusType;
use usb_device::prelude::*;
use usbd_hid::descriptor::{MouseReport, SerializedDescriptor};
use usbd_hid::hid_class::HIDClass;

// shared test VID/PID for HID mice, not for products
const VID_PID: UsbVidPid = UsbVidPid(0x16c0, 0x27da);
const POLL_MS: u8 = 10;

// trackpad on top, buttons below
const PAD: Rectangle = Rectangle::new(Point::new(0, 0), Size::new(240, 248));
const LEFT: Rectangle = Rectangle::new(Point::new(0, 252), Size::new(118, 68));
const RIGHT: Rectangle = Rectangle::new(Point::new(122, 252), Size::new(118, 68));

const LEFT_BUTTON: u8 = 1;
const RIGHT_BUTTON: u8 = 2;

/// Mouse counts per display pixel
const SPEED: i32 = 2;
/// Smallest movement in pixels that moves the cursor, against the touch panel's jitter
const MIN_STEP: i32 = 2;

const OUTLINE: Rgb565 = Rgb565::new(12, 24, 12);
const BUTTON: Rgb565 = Rgb565::new(6, 12, 6);
const BUTTON_PRESSED: Rgb565 = Rgb565::new(18, 36, 18);

// polled from the USB interrupt: the host gives up on a device that answers too slowly
static USB_DEVICE: Mutex<RefCell<Option<UsbDevice<'static, UsbBusType>>>> =
    Mutex::new(RefCell::new(None));
static MOUSE: Mutex<RefCell<Option<HIDClass<'static, UsbBusType>>>> =
    Mutex::new(RefCell::new(None));

#[interrupt]
fn USB() {
    cortex_m::interrupt::free(|cs| {
        let mut device = USB_DEVICE.borrow(cs).borrow_mut();
        let mut mouse = MOUSE.borrow(cs).borrow_mut();
        if let (Some(device), Some(mouse)) = (device.as_mut(), mouse.as_mut()) {
            device.poll(&mut [mouse]);
        }
    });
}

/// Builds the mouse and the USB device and lets the USB interrupt poll them
fn start_usb(usb: &'static ApusUsbBus) {
    let mouse = HIDClass::new_ep_in(usb, MouseReport::desc(), POLL_MS);
    let device = UsbDeviceBuilder::new(usb, VID_PID)
        .manufacturer("HSD")
        .product("APUS trackpad")
        .serial_number("APUS-RS")
        .build();

    cortex_m::interrupt::free(|cs| {
        *MOUSE.borrow(cs).borrow_mut() = Some(mouse);
        *USB_DEVICE.borrow(cs).borrow_mut() = Some(device);
    });

    // SAFETY: the handler only uses the statics, which are set up above
    unsafe { NVIC::unmask(Interrupt::USB) };
}

/// Hands a report to the mouse, false if the previous one is still waiting for the host
fn send(report: &MouseReport) -> bool {
    cortex_m::interrupt::free(|cs| {
        MOUSE
            .borrow(cs)
            .borrow()
            .as_ref()
            .is_some_and(|mouse| mouse.push_input(report).is_ok())
    })
}

/// Relative mouse movement from the touch points on the pad
struct Motion {
    previous: Option<Point>,
    /// mouse counts not sent yet
    pending: Point,
}

impl Motion {
    const fn new() -> Self {
        Self {
            previous: None,
            pending: Point::zero(),
        }
    }

    /// Adds the movement since the last touch point, as long as the finger stays on the pad
    fn track(&mut self, touched: Option<Point>) {
        let on_pad = touched.filter(|point| PAD.contains(*point));
        match (on_pad, self.previous) {
            (Some(point), Some(last)) => {
                let step = point - last;
                if step.x.abs() + step.y.abs() >= MIN_STEP {
                    self.pending += step * SPEED;
                    self.previous = Some(point);
                }
            }
            (point, _) => self.previous = point,
        }
    }

    fn is_pending(&self) -> bool {
        self.pending != Point::zero()
    }

    /// As much of the pending movement as one report holds
    fn step(&self) -> (i8, i8) {
        (
            self.pending.x.clamp(-127, 127) as i8,
            self.pending.y.clamp(-127, 127) as i8,
        )
    }

    fn sent(&mut self, (x, y): (i8, i8)) {
        self.pending -= Point::new(x as i32, y as i32);
    }
}

/// The mouse buttons held by a touch point on the button row
fn button_at(touched: Option<Point>) -> u8 {
    match touched {
        Some(point) if LEFT.contains(point) => LEFT_BUTTON,
        Some(point) if RIGHT.contains(point) => RIGHT_BUTTON,
        _ => 0,
    }
}

fn draw_button(display: &mut ApusDisplay, area: &Rectangle, label: &str, pressed: bool) {
    let fill = if pressed { BUTTON_PRESSED } else { BUTTON };
    let _ = area
        .into_styled(PrimitiveStyle::with_fill(fill))
        .draw(display);
    let style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
    let _ = Text::with_alignment(
        label,
        area.center() + Point::new(0, 7),
        style,
        Alignment::Center,
    )
    .draw(display);
}

fn draw_buttons(display: &mut ApusDisplay, buttons: u8) {
    draw_button(display, &LEFT, "LEFT", buttons == LEFT_BUTTON);
    draw_button(display, &RIGHT, "RIGHT", buttons == RIGHT_BUTTON);
}

fn draw_layout(display: &mut ApusDisplay) {
    let _ = PAD
        .into_styled(PrimitiveStyle::with_stroke(OUTLINE, 2))
        .draw(display);
    draw_buttons(display, 0);
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("APUS-RS example: usb_trackpad");

    let p = pac::Peripherals::take().unwrap();
    let mut board = Board::new(p);

    rprintln!("touch each marker as it appears");
    board
        .touch
        .calibrate_from_display(&mut board.display)
        .unwrap();

    draw_layout(&mut board.display);
    start_usb(board.usb);

    let mut motion = Motion::new();
    let mut buttons = 0u8;
    let mut sent_buttons = 0u8;

    loop {
        let touched = match board.touch.read() {
            Ok(Some(tp)) => Some(Point::new(tp.x as i32, tp.y as i32)),
            _ => None,
        };

        let pressed = button_at(touched);
        if pressed != buttons {
            draw_buttons(&mut board.display, pressed);
            buttons = pressed;
        }

        motion.track(touched);

        if motion.is_pending() || buttons != sent_buttons {
            let (x, y) = motion.step();
            let report = MouseReport {
                buttons,
                x,
                y,
                wheel: 0,
                pan: 0,
            };
            if send(&report) {
                motion.sent((x, y));
                sent_buttons = buttons;
            }
        }
    }
}
