// Shared example helpers; each example uses a subset.
#![allow(dead_code)]

use apus_rs::bsp::buzzer::Buzzer;
use apus_rs::bsp::display::{ApusDisplay, DISPLAY_HEIGHT};
use apus_rs::prelude::*;
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::{Point, Size},
    primitives::Rectangle,
};

/// Update a horizontal bar that grows rightward from x = 0, drawing only the
/// delta area. Widths are in pixels; `fg` fills when growing, `bg` erases when
/// shrinking; `y` is the top edge and `height` the bar thickness.
pub fn hbar_draw_delta(
    display: &mut ApusDisplay,
    prev_w: u16,
    curr_w: u16,
    fg: Rgb565,
    bg: Rgb565,
    y: i32,
    height: u32,
) {
    match curr_w.cmp(&prev_w) {
        core::cmp::Ordering::Greater => {
            let x = prev_w as i32;
            let w = (curr_w - prev_w) as u32;
            let _ = display.fill_solid(&Rectangle::new(Point::new(x, y), Size::new(w, height)), fg);
        }
        core::cmp::Ordering::Less => {
            let x = curr_w as i32;
            let w = (prev_w - curr_w) as u32;
            let _ = display.fill_solid(&Rectangle::new(Point::new(x, y), Size::new(w, height)), bg);
        }
        core::cmp::Ordering::Equal => (),
    }
}

/// Bit-bang a square wave on the GPIO-only buzzer to sound a pitch.
pub fn play_tone(buzzer: &mut Buzzer, delay: &mut impl DelayNs, freq_hz: u32, duration_ms: u32) {
    let half_period_us = 500_000 / freq_hz; // 1_000_000 / (2 * freq_hz)
    let cycles = (duration_ms * 1000) / (half_period_us * 2);
    for _ in 0..cycles {
        buzzer.on();
        delay.delay_us(half_period_us);
        buzzer.off();
        delay.delay_us(half_period_us);
    }
}

/// Update a vertical bar that grows upward from the bottom edge, drawing only
/// the delta area. Heights are in pixels; `fg` fills when growing, `bg` erases
/// when shrinking; `x` is the left edge and `width` the bar thickness.
pub fn vbar_draw_delta(
    display: &mut ApusDisplay,
    prev_h: u16,
    curr_h: u16,
    fg: Rgb565,
    bg: Rgb565,
    x: i32,
    width: u32,
) {
    let bottom = DISPLAY_HEIGHT as i32;
    match curr_h.cmp(&prev_h) {
        core::cmp::Ordering::Greater => {
            let y = bottom - curr_h as i32;
            let h = (curr_h - prev_h) as u32;
            let _ = display.fill_solid(&Rectangle::new(Point::new(x, y), Size::new(width, h)), fg);
        }
        core::cmp::Ordering::Less => {
            let y = bottom - prev_h as i32;
            let h = (prev_h - curr_h) as u32;
            let _ = display.fill_solid(&Rectangle::new(Point::new(x, y), Size::new(width, h)), bg);
        }
        core::cmp::Ordering::Equal => (),
    }
}
