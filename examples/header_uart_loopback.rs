#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use nb::block;
use panic_probe as _;

use apus::board::{defaults::RGB_PWM_FREQUENCY, resources::Resources};
use apus::bsp::{delay::BusyDelay, leds::Leds};
use apus::peripherals::serial::usart1::{self, Usart1Pins};
use apus::prelude::*;

use apus::hal::{pac, prelude::*, time::Bps};

// jumper D1 (TX) to D0 (RX)
const BAUD_RATE: Bps = Bps(115_200);
const PERIOD_MS: u32 = 1000;

#[entry]
fn main() -> ! {
    defmt::info!("APUS-RS example: header_uart_loopback");

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
    let pins = Usart1Pins {
        tx: header.d1,
        rx: header.d0,
    };
    let mut uart = usart1::init(res.peripherals.free.usart1, pins, BAUD_RATE, &mut res.rcc);

    let mut counter: u8 = 0;

    loop {
        // the receiver holds a single byte: send one and read it back before the next
        block!(uart.write(counter)).unwrap();
        block!(uart.flush()).unwrap();

        leds.main_leds_mut().into_iter().for_each(|led| led.off());

        // green on a match, yellow on a wrong or missing byte (no jumper), red on a receive error
        match uart.read() {
            Ok(received) if received == counter => {
                defmt::info!("received {}", received);
                leds.led5.on();
            }
            Ok(received) => {
                defmt::error!("sent {}, received {}", counter, received);
                leds.led4.on();
            }
            Err(nb::Error::WouldBlock) => {
                defmt::error!("sent {}, received nothing", counter);
                leds.led4.on();
            }
            Err(nb::Error::Other(error)) => {
                defmt::error!("receive error: {}", defmt::Debug2Format(&error));
                leds.led3.on();
            }
        }

        delay.delay_ms(PERIOD_MS / 2);
        leds.led5.off();
        delay.delay_ms(PERIOD_MS / 2);

        counter = counter.wrapping_add(1);
    }
}
