#![no_std]
#![no_main]

use cortex_m::asm::delay;
use cortex_m_rt::entry;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point, RgbColor, Size},
    primitives::Rectangle,
};
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

use crate::bsp::leds::led::LedLike;
use embedded_hal_compat::ForwardCompat;
use mipidsi::interface::{Generic8BitBus, ParallelInterface};
use mipidsi::{models::ST7789, Builder};
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
    // split GPIOE/F for the display
    let gpioe = p.GPIOE.split(&mut rcc);
    let gpiof = p.GPIOF.split(&mut rcc);
    // split GPIOC for the LEDs
    let gpioc = p.GPIOC.split(&mut rcc);

    // Do the pin mode changes and build the parallel bus inside a critical section.
    let (di, rst_pin) = cortex_m::interrupt::free(|cs| {
        let db0 = gpioe.pe0.into_push_pull_output(cs);
        let db1 = gpioe.pe1.into_push_pull_output(cs);
        let db2 = gpioe.pe2.into_push_pull_output(cs);
        let db3 = gpioe.pe3.into_push_pull_output(cs);
        let db4 = gpioe.pe4.into_push_pull_output(cs);
        let db5 = gpioe.pe5.into_push_pull_output(cs);
        let db6 = gpioe.pe6.into_push_pull_output(cs);
        let db7 = gpioe.pe7.into_push_pull_output(cs);

        let dc = gpioe.pe9.into_push_pull_output(cs);
        let wr = gpioe.pe10.into_push_pull_output(cs);
        let rst = gpiof.pf10.into_push_pull_output(cs);

        let bus = Generic8BitBus::new((
            db0.forward(),
            db1.forward(),
            db2.forward(),
            db3.forward(),
            db4.forward(),
            db5.forward(),
            db6.forward(),
            db7.forward(),
        ));

        let di = ParallelInterface::new(bus, dc.forward(), wr.forward());
        (di, rst)
    });

    // led construction
    let mut leds = Leds::new(gpioc);

    // delay for display initialisation
    let sys_clock_hz = rcc.clocks.sysclk().0;
    let delay_display = cortex_m::delay::Delay::new(cp.SYST, sys_clock_hz);
    // display construction
    let mut display = Builder::new(ST7789, di)
        // .reset_pin(rst_pin.forward())
        .display_size(DISPLAY_WIDTH, DISPLAY_HEIGHT)
        .init(&mut delay_display.forward())
        .unwrap();
    // display clear
    let _ = display.clear(Rgb565::BLACK);

    let _ = display.fill_solid(
        &Rectangle::new(
            Point::new(0, 0),
            Size::new((DISPLAY_WIDTH / 2) as u32, (DISPLAY_HEIGHT / 2) as u32),
        ),
        Rgb565::RED,
    );

    // led clear
    for led in leds.all_mut().iter_mut() {
        led.off();
    }

    // main loop
    loop {
        rprintln!("Echo...");

        leds.led3.on();
        leds.led4.on();
        leds.led5.on();

        delay(sys_clock_hz / 2);

        leds.led3.off();
        leds.led4.off();
        leds.led5.off();

        delay(sys_clock_hz / 2);
    }
}
