#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_graphics::{
    mono_font::{iso_8859_13::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point, RgbColor, Size},
    primitives::Rectangle,
    text::Text,
    Drawable,
};
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

use crate::bsp::{
    buttons::{button::ButtonLike, Buttons},
    display::{display_init, ApusDisplay, ApusDisplayPins},
    leds::led::LedLike,
    potentiometer::Potentiometer,
};
use stm32f0xx_hal::{adc::Adc, pac, prelude::*};

mod bsp;
use bsp::leds::Leds;

const DISPLAY_HEIGHT: u16 = 320;
const DISPLAY_WIDTH: u16 = 240;

const POTI_THRESHOLD_PERCENT: u16 = 5;
const POTI_MAX_VALUE: u16 = 3300;

#[entry]
fn main() -> ! {
    // rtt
    rtt_init_print!();
    rprintln!("Hello World!");

    // peripherals
    let mut p = pac::Peripherals::take().unwrap();
    let cp = pac::CorePeripherals::take().unwrap();
    // configure clocks
    let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);
    // split GPIOA and GPIOD for the buttons
    let gpioa = p.GPIOA.split(&mut rcc);
    let gpiod = p.GPIOD.split(&mut rcc);
    // split GPIOC for the LEDs
    let gpioc = p.GPIOC.split(&mut rcc);
    // split GPIOE/F for the display
    let gpioe = p.GPIOE.split(&mut rcc);
    let gpiof = p.GPIOF.split(&mut rcc);

    // delay for display initialisation
    let sys_clock_hz = rcc.clocks.sysclk().0;
    let delay_display = cortex_m::delay::Delay::new(cp.SYST, sys_clock_hz);

    // display init
    let display_pins = ApusDisplayPins {
        db0: gpioe.pe0,
        db1: gpioe.pe1,
        db2: gpioe.pe2,
        db3: gpioe.pe3,
        db4: gpioe.pe4,
        db5: gpioe.pe5,
        db6: gpioe.pe6,
        db7: gpioe.pe7,
        n_cs: gpioe.pe8,
        d_c: gpioe.pe9,
        n_wr: gpioe.pe10,
        n_rd: gpioe.pe11,
        n_reset: gpiof.pf10,
    };

    let mut display = display_init(display_pins, delay_display);
    // display clear
    let _ = display.clear(Rgb565::BLACK);

    // led construction
    let mut leds = Leds::new(
        gpioc.pc0, gpioc.pc1, gpioc.pc2, gpioc.pc6, gpioc.pc7, gpioc.pc8,
    );

    // button construction
    let buttons = Buttons::new(gpioa.pa0, gpiod.pd0, gpiod.pd1);

    // poti construction
    let adc = Adc::new(p.ADC, &mut rcc);
    let mut potentiometer = Potentiometer::new(gpioc.pc4, adc);

    let _ = display.fill_solid(
        &Rectangle::new(
            Point::new(0, 0),
            Size::new((DISPLAY_WIDTH) as u32, (DISPLAY_HEIGHT) as u32),
        ),
        Rgb565::WHITE,
    );

    // text
    let style = MonoTextStyle::new(&FONT_10X20, Rgb565::BLACK);
    let _ = Text::new("APUS-RS", Point::new(10, 20), style).draw(&mut display);

    // variables
    let mut poti_prev = 0;

    // main loop
    loop {
        if buttons.wake_up.pressed() {
            leds.led3.on();
        } else {
            leds.led3.off();
        }

        if buttons.user_0.pressed() {
            leds.led4.on();
        } else {
            leds.led4.off();
        }

        if buttons.user_1.pressed() {
            leds.led5.on();
        } else {
            leds.led5.off();
        }

        let poti_value = potentiometer.read();
        let poti_diff = poti_value.abs_diff(poti_prev);
        if poti_diff * 100 / POTI_MAX_VALUE >= POTI_THRESHOLD_PERCENT {
            draw_poti(&mut display, poti_value);
            poti_prev = poti_value;
        }
    }
}

fn draw_poti(display: &mut ApusDisplay, poti_value: u16) {
    // clear bar area
    let _ = display.fill_solid(
        &Rectangle::new(Point::new(0, 30), Size::new(DISPLAY_WIDTH as u32, 30)),
        Rgb565::WHITE,
    );

    // map potentiometer value to display width
    let bar_width = (poti_value as u32 * DISPLAY_WIDTH as u32 / POTI_MAX_VALUE as u32) as u16;

    // draw bar as a filled rectangle
    let _ = display.fill_solid(
        &Rectangle::new(Point::new(0, 30), Size::new(bar_width as u32, 30)),
        Rgb565::BLUE, // Change color as desired
    );
}
