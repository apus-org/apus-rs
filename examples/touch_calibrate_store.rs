#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus_rs::bsp::{
    delay::BusyDelay,
    display::{self, ApusDisplayPins, DisplaySize, DISPLAY_HEIGHT, DISPLAY_WIDTH},
    rtc,
    touch::{self, Touch, TouchCalibrationData},
};
use apus_rs::peripherals::{i2c::i2c1, spi::spi1};

use embedded_hal::delay::DelayNs;
use embedded_hal_bus::spi::ExclusiveDevice;
use stm32f0xx_hal::{pac, prelude::*};

// EEPROM record layout: magic "APUS" followed by the serialized calibration.
// The MCP79410 EEPROM array is 0x00..=0x7F with 8-byte pages, so the 16-byte
// record occupies two pages and must be written one page at a time
const MAGIC: [u8; 4] = *b"APUS";
const RECORD_LEN: usize = MAGIC.len() + TouchCalibrationData::SERIALIZED_LEN;
const EEPROM_BASE: u8 = 0x00;
const PAGE_SIZE: usize = 8;
const WRITE_CYCLE_MS: u32 = 10;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("APUS-RS example: touch_calibrate_store");

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

    // touch controller on SPI1
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

    // MCP79410 EEPROM/RTC
    let i2c1_bus = i2c1::init(p.I2C1, gpiob.pb6, gpiob.pb7, &mut rcc);
    let mut eeprom = rtc::new(i2c1_bus);

    let mut delay = BusyDelay::new(sys_clock_hz);

    // Run the on-screen calibration.
    rprintln!("touch each marker as it appears");
    let calibration = touch.calibrate_from_display(&mut display);
    rprintln!("calibration complete");

    // Assemble the record: magic + serialized calibration.
    let mut record = [0u8; RECORD_LEN];
    record[..MAGIC.len()].copy_from_slice(&MAGIC);
    record[MAGIC.len()..].copy_from_slice(&calibration.to_bytes());

    // write page by page; EEPROM needs its write cycle to finish between writes.
    for (i, page) in record.chunks(PAGE_SIZE).enumerate() {
        let address = EEPROM_BASE + (i * PAGE_SIZE) as u8;
        if eeprom.write_eeprom_data(address, page).is_err() {
            rprintln!("eeprom write failed at 0x{:02X}", address);
            break;
        }
        delay.delay_ms(WRITE_CYCLE_MS);
    }

    // read back to confirm
    let mut verify = [0u8; RECORD_LEN];
    match eeprom.read_eeprom_data(EEPROM_BASE, &mut verify) {
        Ok(()) if verify == record => rprintln!("stored and verified"),
        Ok(()) => rprintln!("verify mismatch"),
        Err(_) => rprintln!("eeprom read-back failed"),
    }

    loop {
        cortex_m::asm::wfi();
    }
}
