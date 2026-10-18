#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus::board::{resources::Resources, I2C1_FREQUENCY, SPI1_FREQUENCY};
use apus::bsp::{
    delay::BusyDelay,
    display::{self, DisplaySize, DISPLAY_HEIGHT, DISPLAY_WIDTH},
    rtc,
    touch::{self, Touch, TouchCalibrationData},
};
use apus::peripherals::{i2c::i2c1, spi::spi1};
use apus::prelude::*;

use embedded_hal_bus::spi::ExclusiveDevice;
use stm32f0xx_hal::pac;

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

    let p = pac::Peripherals::take().unwrap();

    let mut res = Resources::new(p);

    let sys_clock_hz = res.rcc.clocks.sysclk().0;

    // display
    let mut display = display::init(res.display, BusyDelay::new(sys_clock_hz));

    // touch controller on SPI1
    let (spi_bus, cs_pin) =
        spi1::init(res.peripherals.spi1, res.spi1, SPI1_FREQUENCY, &mut res.rcc);
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
    let i2c1_bus = i2c1::init(res.peripherals.i2c1, res.i2c1, I2C1_FREQUENCY, &mut res.rcc);
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
