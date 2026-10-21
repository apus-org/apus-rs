#![no_std]
#![no_main]

use core::convert::Infallible;
use core::ops::ControlFlow;

use cortex_m_rt::entry;
use embedded_hal::digital;
use embedded_hal::spi::SpiBus;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{Mode, SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use panic_rtt_target as _;
use rtt_target::{rprint, rprintln, rtt_init_print};

use apus::board::resources::Resources;
use apus::bsp::delay::BusyDelay;
use apus::peripherals::spi::spi2::{self, Spi2Pins};

use apus::hal::{pac, prelude::*, time::KiloHertz};

// W5500 Ethernet Shield: SD card CS on D4, W5500 CS on D10
// cards start in SPI mode at up to 400 kHz (APB clock / 128)
const SPI2_FREQUENCY: KiloHertz = KiloHertz(375);

const FILE_NAME: &str = "APUS.TXT";
const LINE: &[u8] = b"hello from APUS\r\n";

/// Every file gets the same timestamp
struct FixedTime;

impl TimeSource for FixedTime {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp::from_calendar(2026, 1, 1, 0, 0, 0).unwrap()
    }
}

/// Stands in for the card's CS, which stays low: the card drops the first bit of a reply when
/// CS goes high between the command and the reply, as SpiDevice does between calls
struct Selected;

impl digital::ErrorType for Selected {
    type Error = Infallible;
}

impl digital::OutputPin for Selected {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("APUS-RS example: shield_ethernet_sd_card");

    let p = pac::Peripherals::take().unwrap();
    let mut res = Resources::new(p);
    let sys_clock_hz = res.rcc.clocks.sysclk().0;

    let header = res.header;
    let (mut sd_cs, mut w5500_cs) = cortex_m::interrupt::free(|cs| {
        (
            header.d4.into_push_pull_output(cs),
            header.d10.into_push_pull_output(cs),
        )
    });
    // the W5500 shares the bus and must stay deselected
    w5500_cs.set_high().ok();

    let spi2_pins = Spi2Pins {
        sck: header.d13,
        miso: header.d12,
        mosi: header.d11,
    };
    let mut spi2 = spi2::init(
        res.peripherals.free.spi2,
        spi2_pins,
        SPI2_FREQUENCY,
        &mut res.rcc,
    );

    // at least 74 clocks with CS high put the card into SPI mode
    SpiBus::write(&mut spi2, &[0xFF; 10]).unwrap();
    SpiBus::flush(&mut spi2).unwrap();
    sd_cs.set_low().ok();

    let sd_device = ExclusiveDevice::new_no_delay(spi2, Selected).unwrap();
    let sd_card = SdCard::new(sd_device, BusyDelay::new(sys_clock_hz));
    let card_type = sd_card.get_card_type().expect("no SD card answered");
    rprintln!("card: {:?}", card_type);

    let volume_manager = VolumeManager::new(sd_card, FixedTime);
    let volume = volume_manager.open_volume(VolumeIdx(0)).unwrap();
    let root = volume.open_root_dir().unwrap();

    rprintln!("/");
    root.iterate_dir(|entry| {
        rprintln!("  {:12} {:9} {}", entry.name, entry.size, entry.mtime);
        ControlFlow::Continue(())
    })
    .unwrap();

    let file = root
        .open_file_in_dir(FILE_NAME, Mode::ReadWriteCreateOrAppend)
        .unwrap();
    file.write(LINE).unwrap();
    file.close().unwrap();

    let file = root.open_file_in_dir(FILE_NAME, Mode::ReadOnly).unwrap();
    rprintln!("{} ({} bytes):", FILE_NAME, file.length());
    let mut buf = [0u8; 64];
    while !file.is_eof() {
        let len = file.read(&mut buf).unwrap();
        rprint!("{}", core::str::from_utf8(&buf[..len]).unwrap_or("?"));
    }
    file.close().unwrap();

    rprintln!("done");
    loop {
        cortex_m::asm::wfi();
    }
}
