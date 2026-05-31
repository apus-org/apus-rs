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
    rtc,
    touch::{self, Touch, TouchCalibrationData},
};
use apus_rs::peripherals::{i2c::i2c1, spi::spi1};

use embedded_hal_bus::spi::ExclusiveDevice;
use stm32f0xx_hal::{pac, prelude::*};

// Must match touch_calibrate_store.
const MAGIC: [u8; 4] = *b"APUS";
const RECORD_LEN: usize = MAGIC.len() + TouchCalibrationData::SERIALIZED_LEN;
const EEPROM_BASE: u8 = 0x00;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("APUS-RS example: touch_calibrate_restore");

    let mut p = pac::Peripherals::take().unwrap();

    let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);

    let gpiob = p.GPIOB.split(&mut rcc);
    let gpioe = p.GPIOE.split(&mut rcc);
    let gpiof = p.GPIOF.split(&mut rcc);

    let sys_clock_hz = rcc.clocks.sysclk().0;

    // display
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
    let mut display = display::init(display_pins, BusyDelay::new(sys_clock_hz));

    // Touch controller on SPI1.
    let (spi_bus, cs_pin) = spi1::init(
        p.SPI1, gpioe.pe12, gpioe.pe13, gpioe.pe14, gpioe.pe15, &mut rcc,
    );
    let spi_dev = ExclusiveDevice::new(spi_bus, cs_pin, BusyDelay::new(sys_clock_hz)).unwrap();
    let mut touch = Touch::new(
        spi_dev,
        touch::DEFAULT_THRESHOLD,
        DisplaySize {
            x: DISPLAY_WIDTH,
            y: DISPLAY_HEIGHT,
        },
    );

    // MCP79410 EEPROM/RTC on I2C1.
    let i2c1_bus = i2c1::init(p.I2C1, gpiob.pb6, gpiob.pb7, &mut rcc);
    let mut eeprom = rtc::new(i2c1_bus);

    // Read the record (a sequential read may span pages freely).
    let mut record = [0u8; RECORD_LEN];
    if eeprom.read_eeprom_data(EEPROM_BASE, &mut record).is_err() {
        rprintln!("eeprom read failed");
        idle();
    }
    if record[..MAGIC.len()] != MAGIC {
        rprintln!("no calibration stored, run touch_calibrate_store first");
        idle();
    }

    let mut payload = [0u8; TouchCalibrationData::SERIALIZED_LEN];
    payload.copy_from_slice(&record[MAGIC.len()..]);
    touch.calibrate_from_data(TouchCalibrationData::from_bytes(&payload));
    rprintln!("calibration restored from EEPROM");

    const PAINT_COLOR: Rgb565 = Rgb565::WHITE;
    const PAINT_SIZE: u32 = 10;
    const STYLE: PrimitiveStyle<Rgb565> = PrimitiveStyle::with_fill(PAINT_COLOR);

    loop {
        if let Ok(Some(tp)) = touch.read() {
            rprintln!("touch: x={} y={} z={:.2}", tp.x, tp.y, tp.z);
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
