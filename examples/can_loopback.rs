#![no_std]
#![no_main]

use cortex_m_rt::entry;
use nb::block;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use apus::board::{resources::Resources, RGB_PWM_FREQUENCY};
use apus::bsp::{delay::BusyDelay, leds::Leds};
use apus::peripherals::can::{self, CanBus};
use apus::prelude::*;

use apus::hal::can::bxcan::{filter::Mask32, Data, Fifo, Frame, StandardId};
use apus::hal::pac;

// 16 time quanta per bit, sampled after 1 + 13 of them (87.5 %)
const BIT_RATE: u32 = 500_000;
const SEGMENT_1: u32 = 13;
const SEGMENT_2: u32 = 2;
const JUMP_WIDTH: u32 = 1;
const QUANTA_PER_BIT: u32 = 1 + SEGMENT_1 + SEGMENT_2;

/// Value of the bit timing register (BTR) for [`BIT_RATE`] from the APB clock
fn bit_timing(pclk_hz: u32) -> u32 {
    let prescaler = pclk_hz / (BIT_RATE * QUANTA_PER_BIT);
    (JUMP_WIDTH - 1) << 24 | (SEGMENT_2 - 1) << 20 | (SEGMENT_1 - 1) << 16 | (prescaler - 1)
}

const FRAME_ID: u16 = 0x123;
const PERIOD_MS: u32 = 1000;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("APUS-RS example: can_loopback");

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

    let pclk_hz = res.rcc.clocks.pclk().0;
    let (can_instance, can_mode) = can::init(res.peripherals.can, res.can, &mut res.rcc);
    let mut bus = CanBus::new(can_instance, can_mode);
    let can = &mut bus.controller;

    // loopback + silent: frames stay inside the controller and the bus is not driven
    can.modify_config()
        .set_bit_timing(bit_timing(pclk_hz))
        .set_loopback(true)
        .set_silent(true)
        .leave_disabled();
    can.modify_filters()
        .enable_bank(0, Fifo::Fifo0, Mask32::accept_all());
    block!(can.enable_non_blocking()).unwrap();

    let id = StandardId::new(FRAME_ID).unwrap();
    let mut counter: u8 = 0;

    loop {
        let sent = Frame::new_data(id, Data::new(&[counter]).unwrap());
        block!(can.transmit(&sent)).unwrap();

        leds.main_leds_mut().into_iter().for_each(|led| led.off());

        // green on a matching frame, yellow on a mismatch, red on an overrun
        match block!(can.receive()) {
            Ok(received) if received == sent => {
                rprintln!("received {}", counter);
                leds.led5.on();
            }
            Ok(received) => {
                rprintln!("mismatch: {:?}", received);
                leds.led4.on();
            }
            Err(_) => {
                rprintln!("receive overrun");
                leds.led3.on();
            }
        }

        delay.delay_ms(PERIOD_MS / 2);
        leds.led5.off();
        delay.delay_ms(PERIOD_MS / 2);

        counter = counter.wrapping_add(1);
    }
}
