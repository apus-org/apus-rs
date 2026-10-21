#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{Point, RgbColor, Size};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use panic_probe as _;

use apus::board::{defaults::SPI1_FREQUENCY, resources::Resources};
use apus::bsp::{
    delay::BusyDelay,
    display::{self, DisplaySize, DISPLAY_HEIGHT, DISPLAY_WIDTH},
    touch::{self, Touch},
};
use apus::peripherals::spi::spi1;

use apus::hal::pac;

#[entry]
fn main() -> ! {
    defmt::info!("APUS-RS example: touch");

    // Peripherals
    let p = pac::Peripherals::take().unwrap();
    let cp = pac::CorePeripherals::take().unwrap();

    let mut res = Resources::new(p);

    // Initialize display
    let sys_clock_hz = res.rcc.clocks.sysclk().0;
    let delay_display = BusyDelay::new(sys_clock_hz);

    let mut display = display::init(res.display, delay_display);

    // Initialize SPI1 and get the bus and CS pin
    let (spi_bus, cs_pin) =
        spi1::init(res.peripherals.spi1, res.spi1, SPI1_FREQUENCY, &mut res.rcc);

    // Create a small blocking delay helper for the main loop
    let sys_clock_hz = res.rcc.clocks.sysclk().0;
    let mut delay = cortex_m::delay::Delay::new(cp.SYST, sys_clock_hz);

    // Touch driver owns the SPI1 bus exclusively
    let spi_dev =
        embedded_hal_bus::spi::ExclusiveDevice::new(spi_bus, cs_pin, BusyDelay::new(sys_clock_hz))
            .unwrap();

    // Initialize the BSP touch driver from the prepared SPI device
    let mut touch = Touch::new(
        spi_dev,
        res.touch,
        touch::DEFAULT_THRESHOLD,
        DisplaySize {
            x: DISPLAY_WIDTH,
            y: DISPLAY_HEIGHT,
        },
    );

    const PAINT_COLOR: Rgb565 = Rgb565::WHITE;
    const PAINT_SIZE: u32 = 10;
    const STYLE: PrimitiveStyle<Rgb565> = PrimitiveStyle::with_fill(PAINT_COLOR);

    // Calibrate touch using the display
    defmt::info!("Starting touch calibration...");
    defmt::info!("Please touch each marker as it appears");
    touch.calibrate_from_display(&mut display).unwrap();
    defmt::info!("Calibration complete!");

    loop {
        match touch.read() {
            Ok(Some(tp)) => {
                defmt::info!("touch: x={} y={} z={}", tp.x, tp.y, tp.z);
                let _ = Rectangle::new(
                    Point::new(tp.x as i32, tp.y as i32),
                    Size::new(PAINT_SIZE, PAINT_SIZE),
                )
                .draw_styled(&STYLE, &mut display);
            }
            Ok(None) => {
                defmt::info!("no touch");
                delay.delay_ms(200);
            }
            Err(e) => {
                defmt::error!("error reading touch: {}", defmt::Debug2Format(&e));
                delay.delay_ms(1000);
            }
        }
    }
}
