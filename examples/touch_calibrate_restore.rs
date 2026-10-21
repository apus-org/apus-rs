#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{Point, RgbColor, Size};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use panic_probe as _;

use apus::board::{resources::Resources, I2C1_FREQUENCY, SPI1_FREQUENCY};
use apus::bsp::{
    delay::BusyDelay,
    display::{self, DisplaySize, DISPLAY_HEIGHT, DISPLAY_WIDTH},
    rtc,
    touch::{self, Touch, TouchCalibrationData},
};
use apus::peripherals::{i2c::i2c1, spi::spi1};

use apus::hal::pac;
use embedded_hal_bus::spi::ExclusiveDevice;

// Must match touch_calibrate_store.
const MAGIC: [u8; 4] = *b"APUS";
const RECORD_LEN: usize = MAGIC.len() + TouchCalibrationData::SERIALIZED_LEN;
const EEPROM_BASE: u8 = 0x00;

#[entry]
fn main() -> ! {
    defmt::info!("APUS-RS example: touch_calibrate_restore");

    let p = pac::Peripherals::take().unwrap();

    let mut res = Resources::new(p);

    let sys_clock_hz = res.rcc.clocks.sysclk().0;

    // display
    let mut display = display::init(res.display, BusyDelay::new(sys_clock_hz));

    // Touch controller on SPI1.
    let (spi_bus, cs_pin) =
        spi1::init(res.peripherals.spi1, res.spi1, SPI1_FREQUENCY, &mut res.rcc);
    let spi_dev = ExclusiveDevice::new(spi_bus, cs_pin, BusyDelay::new(sys_clock_hz)).unwrap();
    let mut touch = Touch::new(
        spi_dev,
        res.touch,
        touch::DEFAULT_THRESHOLD,
        DisplaySize {
            x: DISPLAY_WIDTH,
            y: DISPLAY_HEIGHT,
        },
    );

    // MCP79410 EEPROM/RTC on I2C1.
    let i2c1_bus = i2c1::init(res.peripherals.i2c1, res.i2c1, I2C1_FREQUENCY, &mut res.rcc);
    let mut eeprom = rtc::new(i2c1_bus);

    // Read the record (a sequential read may span pages freely).
    let mut record = [0u8; RECORD_LEN];
    if eeprom.read_eeprom_data(EEPROM_BASE, &mut record).is_err() {
        defmt::error!("eeprom read failed");
        idle();
    }
    if record[..MAGIC.len()] != MAGIC {
        defmt::info!("no calibration stored, run touch_calibrate_store first");
        idle();
    }

    let mut payload = [0u8; TouchCalibrationData::SERIALIZED_LEN];
    payload.copy_from_slice(&record[MAGIC.len()..]);
    touch.calibrate_from_data(TouchCalibrationData::from_bytes(&payload));
    defmt::info!("calibration restored from EEPROM");

    const PAINT_COLOR: Rgb565 = Rgb565::WHITE;
    const PAINT_SIZE: u32 = 10;
    const STYLE: PrimitiveStyle<Rgb565> = PrimitiveStyle::with_fill(PAINT_COLOR);

    loop {
        if let Ok(Some(tp)) = touch.read() {
            defmt::info!("touch: x={} y={} z={}", tp.x, tp.y, tp.z);
            let _ = Rectangle::new(
                Point::new(tp.x as i32, tp.y as i32),
                Size::new(PAINT_SIZE, PAINT_SIZE),
            )
            .draw_styled(&STYLE, &mut display);
        }
    }
}

fn idle() -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}
