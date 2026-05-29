#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point, Primitive, RgbColor, Size},
    primitives::{Circle, PrimitiveStyle, Rectangle, Triangle},
    Drawable,
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus_rs::bsp::delay::BusyDelay;
use apus_rs::bsp::display::{display_init, ApusDisplayPins, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use stm32f0xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    // RTT
    rtt_init_print!();
    rprintln!("APUS-RS example: display demo");

    // Peripherals
    let mut p = pac::Peripherals::take().unwrap();

    // Clocks
    let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);

    // GPIO splits
    let gpioe = p.GPIOE.split(&mut rcc);
    let gpiof = p.GPIOF.split(&mut rcc);

    // Delay for display init
    let sys_clock_hz = rcc.clocks.sysclk().0;
    let delay_display = BusyDelay::new(sys_clock_hz as u32);

    // Display pins
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
    let _ = display.clear(Rgb565::BLACK);

    // Draw a filled rectangle
    let rect = Rectangle::new(Point::new(10, 10), Size::new(100, 60));
    let _ = display.fill_solid(&rect, Rgb565::BLUE);

    // Draw a circle outline
    let circ = Circle::new(Point::new(180, 60), 40)
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 4));
    let _ = circ.draw(&mut display);

    // Draw a filled triangle
    let tri = Triangle::new(
        Point::new(60, 200),
        Point::new(120, 280),
        Point::new(10, 280),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::RED));
    let _ = tri.draw(&mut display);

    // Main loop does nothing, just keeps shapes on screen
    loop {
        cortex_m::asm::wfi();
    }
}
