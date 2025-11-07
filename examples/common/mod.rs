use apus_rs::bsp::display::{ApusDisplay, DISPLAY_WIDTH};
use apus_rs::bsp::potentiometer::MAX_VALUE;
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::{Point, Size},
    primitives::Rectangle,
};

/// Update a horizontal potentiometer bar by only drawing the delta area.
///
/// Parameters:
/// - `display`: the display to draw on
/// - `prev_value`, `curr_value`: raw potentiometer values (0..=POTI_MAX_VALUE)
/// - `fg`: color for the filled bar (when increasing)
/// - `bg`: background color to use when erasing (when decreasing)
/// - `y`: top Y position of the bar
/// - `height`: height of the bar in pixels
pub fn poti_draw_delta(
    display: &mut ApusDisplay,
    prev_value: u16,
    curr_value: u16,
    fg: Rgb565,
    bg: Rgb565,
    y: i32,
    height: u32,
) {
    let prev_w = (prev_value as u32 * DISPLAY_WIDTH as u32 / MAX_VALUE as u32) as u16;
    let curr_w = (curr_value as u32 * DISPLAY_WIDTH as u32 / MAX_VALUE as u32) as u16;

    match curr_w.cmp(&prev_w) {
        core::cmp::Ordering::Greater => {
            let x = prev_w as i32;
            let w = (curr_w - prev_w) as u32;
            if w > 0 {
                let _ =
                    display.fill_solid(&Rectangle::new(Point::new(x, y), Size::new(w, height)), fg);
            }
        }
        core::cmp::Ordering::Less => {
            let x = curr_w as i32;
            let w = (prev_w - curr_w) as u32;
            if w > 0 {
                let _ =
                    display.fill_solid(&Rectangle::new(Point::new(x, y), Size::new(w, height)), bg);
            }
        }
        core::cmp::Ordering::Equal => (),
    }
}
