#![no_std]
#![no_main]

use cortex_m::asm::delay;
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

use crate::bsp::display::{display_init, ApusDisplayPins};
use stm32f0xx_hal::{pac, prelude::*};

mod bsp;
use bsp::leds::Leds;

const DISPLAY_HEIGHT: u16 = 320;
const DISPLAY_WIDTH: u16 = 240;

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
        w_r: gpioe.pe10,
        n_reset: gpiof.pf10,
    };

    let mut display = display_init(display_pins, delay_display);
    // display clear
    let _ = display.clear(Rgb565::BLACK);

    // led construction
    let mut leds = Leds::new(
        gpioc.pc0, gpioc.pc1, gpioc.pc2, gpioc.pc6, gpioc.pc7, gpioc.pc8,
    );

    // main loop
    loop {
        leds.main_leds_mut().iter_mut().for_each(|led| led.on());

        let _ = display.fill_solid(
            &Rectangle::new(
                Point::new(0, 0),
                Size::new((DISPLAY_WIDTH / 2) as u32, (DISPLAY_HEIGHT / 2) as u32),
            ),
            Rgb565::WHITE,
        );

        delay(sys_clock_hz / 2);

        leds.main_leds_mut().iter_mut().for_each(|led| led.off());

        let style = MonoTextStyle::new(&FONT_10X20, Rgb565::BLACK);
        // Create a text at position (20, 30) and draw it using the previously defined style
        let _ = Text::new("APUS-RS", Point::new(10, 20), style).draw(&mut display);

        delay(sys_clock_hz / 2);
    }
}
