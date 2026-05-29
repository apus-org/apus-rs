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
use embedded_hal_compat::ForwardCompat;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus_rs::bsp::{
    buttons::{button::ButtonLike, Buttons},
    display::{self, ApusDisplayPins, DISPLAY_HEIGHT, DISPLAY_WIDTH},
    leds::{led::LedLike, Leds},
    potentiometer::{Potentiometer, MAX_VALUE},
};
use stm32f0xx_hal::{adc::Adc, pac, prelude::*};

mod common;

const POTI_THRESHOLD_PERCENT: u32 = 5;

#[entry]
fn main() -> ! {
    // rtt
    rtt_init_print!();
    rprintln!("APUS-RS example: full_demo");
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

    let mut display = display::init(display_pins, delay_display.forward());
    // display clear
    let _ = display.clear(Rgb565::BLACK);

    // led construction
    let mut leds = Leds::new(
        gpioc.pc0, gpioc.pc1, gpioc.pc2, gpioc.pc6, gpioc.pc7, gpioc.pc8,
    );

    // button construction
    let mut buttons = Buttons::new(gpioa.pa0, gpiod.pd0, gpiod.pd1);

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
    let mut poti_previous_value = 0;

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

        let poti_current_value = potentiometer.read();
        let poti_diff: u32 = poti_current_value.abs_diff(poti_previous_value) as u32;
        if poti_diff.saturating_mul(100) / MAX_VALUE as u32 >= POTI_THRESHOLD_PERCENT {
            // update only the delta area (black bar on white background)
            common::poti_draw_delta(
                &mut display,
                poti_previous_value,
                poti_current_value,
                Rgb565::BLACK,
                Rgb565::WHITE,
                30,
                30,
            );
            poti_previous_value = poti_current_value;
        }
    }
}
