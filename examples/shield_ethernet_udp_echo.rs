#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::{entry, exception};
use defmt_rtt as _;
use embedded_hal_bus::spi::ExclusiveDevice;
use panic_probe as _;

use apus::board::resources::Resources;
use apus::peripherals::spi::spi2::{self, Spi2Pins};

use apus::hal::{pac, prelude::*, time::KiloHertz};
use w5500_dhcp::{
    hl::{Error, Udp},
    ll::{eh1::vdm::W5500, net::Eui48Addr, Registers, Sn},
    Client, Hostname,
};

// W5500 Ethernet Shield: W5500 CS on D10, interrupt on D2, SD card CS on D4
const SPI2_FREQUENCY: KiloHertz = KiloHertz(12_000);

// locally administered MAC address
const MAC_ADDRESS: Eui48Addr = Eui48Addr::new(0x02, 0x00, b'A', b'P', b'U', b'S');
const HOSTNAME: Hostname = Hostname::new_unwrapped("apus");

const DHCP_SN: Sn = Sn::Sn0;
const ECHO_SN: Sn = Sn::Sn1;
const ECHO_PORT: u16 = 7;

// DHCP retries and lease timers count in seconds
static SECONDS: AtomicU32 = AtomicU32::new(0);

#[exception]
fn SysTick() {
    SECONDS.store(SECONDS.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
}

#[entry]
fn main() -> ! {
    defmt::info!("APUS-RS example: shield_ethernet_udp_echo");

    let p = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut res = Resources::new(p);

    // external clock = HCLK / 8, so one second fits the 24-bit reload
    let mut syst = cp.SYST;
    syst.set_clock_source(SystClkSource::External);
    syst.set_reload(res.rcc.clocks.hclk().0 / 8 - 1);
    syst.clear_current();
    syst.enable_interrupt();
    syst.enable_counter();

    let header = res.header;
    let (w5500_cs, mut sd_cs, w5500_int) = cortex_m::interrupt::free(|cs| {
        (
            header.d10.into_push_pull_output(cs),
            header.d4.into_push_pull_output(cs),
            header.d2.into_pull_up_input(cs),
        )
    });
    // the SD card shares the bus and must stay deselected
    sd_cs.set_high().ok();

    let spi2_pins = Spi2Pins {
        sck: header.d13,
        miso: header.d12,
        mosi: header.d11,
    };
    let spi2 = spi2::init(
        res.peripherals.free.spi2,
        spi2_pins,
        SPI2_FREQUENCY,
        &mut res.rcc,
    );
    let mut w5500 = W5500::new(ExclusiveDevice::new_no_delay(spi2, w5500_cs).unwrap());

    w5500.set_shar(&MAC_ADDRESS).unwrap();

    let mut dhcp = Client::new(DHCP_SN, 0, MAC_ADDRESS, HOSTNAME);
    dhcp.setup_socket(&mut w5500).unwrap();
    w5500.udp_bind(ECHO_SN, ECHO_PORT).unwrap();

    let mut next_dhcp_secs = 0;
    let mut link = None;
    let mut leased = None;
    let mut buf = [0u8; 512];

    loop {
        let phy_link = w5500.phycfgr().unwrap().lnk();
        if link != Some(phy_link) {
            link = Some(phy_link);
            defmt::info!("link {}", defmt::Debug2Format(&phy_link));
        }

        let now = SECONDS.load(Ordering::Relaxed);
        if now >= next_dhcp_secs || w5500_int.is_low().unwrap() {
            next_dhcp_secs = now + dhcp.process(&mut w5500, now).unwrap();
        }

        if dhcp.leased_ip() != leased {
            leased = dhcp.leased_ip();
            match leased {
                Some(ip) => defmt::info!(
                    "leased {}, echoing on UDP port {}",
                    defmt::Display2Format(&ip),
                    ECHO_PORT
                ),
                None => defmt::warn!("lease lost"),
            }
        }

        match w5500.udp_recv_from(ECHO_SN, &mut buf) {
            Ok((len, origin)) => {
                defmt::info!("{} bytes from {}", len, defmt::Display2Format(&origin));
                if w5500
                    .udp_send_to(ECHO_SN, &buf[..len.into()], &origin)
                    .is_err()
                {
                    defmt::error!("send error");
                }
            }
            Err(Error::WouldBlock) => {}
            Err(error) => defmt::error!("receive error: {}", defmt::Debug2Format(&error)),
        }
    }
}
