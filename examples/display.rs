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

use apus::board::resources::Resources;
use apus::bsp::delay::BusyDelay;
use apus::bsp::display::{self};
use apus::hal::pac;

#[entry]
fn main() -> ! {
    // RTT
    rtt_init_print!();
    rprintln!("APUS-RS example: display demo");

    // Peripherals
    let p = pac::Peripherals::take().unwrap();

    let res = Resources::new(p);

    // Delay for display init
    let sys_clock_hz = res.rcc.clocks.sysclk().0;
    let delay_display = BusyDelay::new(sys_clock_hz);

    let mut display = display::init(res.display, delay_display);
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
