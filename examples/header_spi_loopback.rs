#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use embedded_hal::spi::SpiBus;
use panic_probe as _;

use apus::board::{defaults::RGB_PWM_FREQUENCY, resources::Resources};
use apus::bsp::{delay::BusyDelay, leds::Leds};
use apus::peripherals::spi::spi2::{self, Spi2Pins};
use apus::prelude::*;

use apus::hal::{pac, time::KiloHertz};

// jumper D11 (MOSI) to D12 (MISO)
const SPI2_FREQUENCY: KiloHertz = KiloHertz(1_000);
const MESSAGE_LENGTH: usize = 4;
const PERIOD_MS: u32 = 1000;

#[entry]
fn main() -> ! {
    defmt::info!("APUS-RS example: header_spi_loopback");

    let p = pac::Peripherals::take().unwrap();
    let mut res = Resources::new(p);

    let sys_clock_hz = res.rcc.clocks.sysclk().0;
    let mut delay = BusyDelay::new(sys_clock_hz);
    let mut leds = Leds::new(
        res.leds,
        res.peripherals.tim3,
        RGB_PWM_FREQUENCY,
        &mut res.rcc,
    );

    let header = res.header;
    let pins = Spi2Pins {
        sck: header.d13,
        miso: header.d12,
        mosi: header.d11,
    };
    let mut spi = spi2::init(
        res.peripherals.free.spi2,
        pins,
        SPI2_FREQUENCY,
        &mut res.rcc,
    );

    let mut counter: u8 = 0;

    loop {
        let sent: [u8; MESSAGE_LENGTH] = core::array::from_fn(|i| counter.wrapping_add(i as u8));
        let mut received = [0; MESSAGE_LENGTH];

        leds.main_leds_mut().into_iter().for_each(|led| led.off());

        // green on a match, yellow on a mismatch (jumper missing), red on a bus error
        match spi.transfer(&mut received, &sent) {
            Ok(()) if received == sent => {
                defmt::info!("received {}", received);
                leds.led5.on();
            }
            Ok(()) => {
                defmt::error!("sent {}, received {}", sent, received);
                leds.led4.on();
            }
            Err(error) => {
                defmt::error!("transfer error: {}", defmt::Debug2Format(&error));
                leds.led3.on();
            }
        }

        delay.delay_ms(PERIOD_MS / 2);
        leds.led5.off();
        delay.delay_ms(PERIOD_MS / 2);

        counter = counter.wrapping_add(1);
    }
}
