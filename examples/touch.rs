#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus_rs::peripherals::spi::spi1::init_spi1;
use apus_rs::bsp::{delay::BusyDelay, touch};

use stm32f0xx_hal::{pac, prelude::*};
use embedded_hal_compat::ForwardCompat;

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

    // GPIOE (SPI1 on PE13/14/15, CS on PE12)
    let gpioe = p.GPIOE.split(&mut rcc);

    // Initialize SPI1 and get a shared-bus manager and CS pin
    let (manager, cs_pin) = init_spi1(p.SPI1, gpioe.pe12, gpioe.pe13, gpioe.pe14, gpioe.pe15, &mut rcc);

    // Create a small blocking delay helper for the main loop
    let sys_clock_hz = rcc.clocks.sysclk().0;
    let mut delay = cortex_m::delay::Delay::new(cp.SYST, sys_clock_hz);

    // Acquire a proxy from the shared-bus manager
    let proxy = manager.acquire_spi();

    // Build an eh1-compatible ExclusiveDevice by forwarding the proxy and CS pin
    let spi_dev = embedded_hal_bus::spi::ExclusiveDevice::new(
        proxy.forward(),
        cs_pin.forward(),
        BusyDelay::new(sys_clock_hz as u32),
    )
    .unwrap();

    // Initialize the BSP touch driver from the prepared SPI device
    let mut touch = touch::init(spi_dev, touch::DEFAULT_THRESHOLD);

    loop {
        match touch.get_touch() {
            Ok(Some(tp)) => rprintln!("touch: x={} y={} z={:.2}", tp.x, tp.y, tp.z),
            Ok(None) => rprintln!("no touch"),
            Err(_e) => rprintln!("spi error reading touch"),
        }

        delay.delay_ms(200u32);
    }
}
