#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{Point, RgbColor, Size};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus_rs::bsp::{
    delay::BusyDelay,
    display::{self, ApusDisplayPins, DisplaySize, DISPLAY_HEIGHT, DISPLAY_WIDTH},
    touch::{self, Touch},
};
use apus_rs::peripherals::spi::spi1;

use stm32f0xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    // RTT
    rtt_init_print!();
    rprintln!("APUS-RS example: touch");

    // Peripherals
    let mut p = pac::Peripherals::take().unwrap();
    let cp = pac::CorePeripherals::take().unwrap();

    // Clocks
    let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);

    // GPIO splits
    let gpioe = p.GPIOE.split(&mut rcc);
    let gpiof = p.GPIOF.split(&mut rcc);

    // Initialize display
    let sys_clock_hz = rcc.clocks.sysclk().0;
    let delay_display = BusyDelay::new(sys_clock_hz as u32);

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

    let mut display = display::init(display_pins, delay_display);

    // Initialize SPI1 and get the bus and CS pin
    let (spi_bus, cs_pin) = spi1::init(
        p.SPI1, gpioe.pe12, gpioe.pe13, gpioe.pe14, gpioe.pe15, &mut rcc,
    );

    // Create a small blocking delay helper for the main loop
    let sys_clock_hz = rcc.clocks.sysclk().0;
    let mut delay = cortex_m::delay::Delay::new(cp.SYST, sys_clock_hz);

    // Touch driver owns the SPI1 bus exclusively
    let spi_dev = embedded_hal_bus::spi::ExclusiveDevice::new(
        spi_bus,
        cs_pin,
        BusyDelay::new(sys_clock_hz as u32),
    )
    .unwrap();

    // Initialize the BSP touch driver from the prepared SPI device
    let mut touch = Touch::new(
        spi_dev,
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
    rprintln!("Starting touch calibration...");
    rprintln!("Please touch each marker as it appears");
    let calibration_data = touch.calibrate_from_display(&mut display);
    touch.calibrate_from_data(calibration_data);
    rprintln!("Calibration complete!");

    loop {
        match touch.read() {
            Ok(Some(tp)) => {
                rprintln!("touch: x={} y={} z={:.2}", tp.x, tp.y, tp.z);
                let _ = Rectangle::new(
                    Point::new(tp.x as i32, tp.y as i32),
                    Size::new(PAINT_SIZE, PAINT_SIZE),
                )
                .draw_styled(&STYLE, &mut display);
            }
            Ok(None) => {
                rprintln!("no touch");
                delay.delay_ms(200u32);
            }
            Err(e) => {
                rprintln!("error reading touch: {:?}", e);
                delay.delay_ms(1000u32);
            }
        }
    }
}
