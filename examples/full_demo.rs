#![no_std]
#![no_main]

use core::fmt::Write;

use cortex_m_rt::entry;
use defmt_rtt as _;
use embedded_graphics::{
    mono_font::{
        iso_8859_13::{FONT_10X20, FONT_6X10},
        MonoTextStyleBuilder,
    },
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point, RgbColor, Size},
    primitives::Rectangle,
    text::Text,
    Drawable,
};
use heapless::String;
use panic_probe as _;

use apus::board::Board;
use apus::bsp::{
    display::{DISPLAY_HEIGHT, DISPLAY_WIDTH},
    potentiometer::MAX_VALUE,
    rtc::{DateTimeAccess, NaiveDate},
};
use apus::prelude::*;

use apus::hal::pac;

mod common;

const POTI_THRESHOLD_PERCENT: u32 = 5;
const LOOP_DELAY_MS: u32 = 50;
const SENSOR_PERIOD_TICKS: u32 = 20;

#[entry]
fn main() -> ! {
    defmt::info!("APUS-RS example: full_demo");

    let p = pac::Peripherals::take().unwrap();
    let mut board = Board::new(p);

    // start sensors
    if board.temperature.sensor.enable().is_err() {
        defmt::error!("failed to enable temperature sensor");
    }
    let datetime = NaiveDate::from_ymd_opt(2002, 6, 10)
        .unwrap()
        .and_hms_opt(9, 0, 0)
        .unwrap();
    if board.rtc.clock.set_datetime(&datetime).is_err() {
        defmt::error!("failed to set datetime");
    }
    if board.rtc.clock.enable().is_err() {
        defmt::error!("failed to start rtc oscillator");
    }

    // text styles
    let title_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(Rgb565::BLACK)
        .background_color(Rgb565::WHITE)
        .build();
    let value_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(Rgb565::BLACK)
        .background_color(Rgb565::WHITE)
        .build();

    // white background + static title
    let _ = board.display.fill_solid(
        &Rectangle::new(
            Point::new(0, 0),
            Size::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32),
        ),
        Rgb565::WHITE,
    );
    let _ = Text::new("APUS-RS", Point::new(10, 20), title_style).draw(&mut board.display);

    let mut poti_previous_value = 0u16;
    let mut tick: u32 = 0;

    loop {
        // leds driven by buttons
        if board.buttons.wake_up.pressed() {
            board.leds.led3.on();
        } else {
            board.leds.led3.off();
        }
        if board.buttons.user_0.pressed() {
            board.leds.led4.on();
        } else {
            board.leds.led4.off();
        }
        if board.buttons.user_1.pressed() {
            board.leds.led5.on();
        } else {
            board.leds.led5.off();
        }

        // potentiometer bar only redrawn on meaningful changes
        let poti_current_value = board.potentiometer.read(&mut board.adc);
        let poti_diff = poti_current_value.abs_diff(poti_previous_value) as u32;
        if poti_diff.saturating_mul(100) / MAX_VALUE as u32 >= POTI_THRESHOLD_PERCENT {
            let prev_w =
                (poti_previous_value as u32 * DISPLAY_WIDTH as u32 / MAX_VALUE as u32) as u16;
            let curr_w =
                (poti_current_value as u32 * DISPLAY_WIDTH as u32 / MAX_VALUE as u32) as u16;
            common::hbar_draw_delta(
                &mut board.display,
                prev_w,
                curr_w,
                Rgb565::BLACK,
                Rgb565::WHITE,
                90,
                30,
            );
            poti_previous_value = poti_current_value;
        }

        // temperature + time
        if tick.is_multiple_of(SENSOR_PERIOD_TICKS) {
            let mut line: String<32> = String::new();
            match board.temperature.sensor.read_temperature() {
                Ok(c) => {
                    let _ = write!(line, "temp:{:7.2} C", c);
                    let _ =
                        Text::new(&line, Point::new(10, 45), value_style).draw(&mut board.display);
                }
                Err(_) => defmt::error!("temp read error"),
            }
            match board.rtc.clock.datetime() {
                Ok(dt) => {
                    line.clear();
                    let _ = write!(line, "time: {}", dt);
                    let _ =
                        Text::new(&line, Point::new(10, 60), value_style).draw(&mut board.display);
                }
                Err(_) => defmt::error!("rtc read error"),
            }
        }

        tick = tick.wrapping_add(1);
        board.delay.delay_ms(LOOP_DELAY_MS);
    }
}
