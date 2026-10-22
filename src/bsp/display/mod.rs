pub mod data_bus;

use cortex_m::interrupt;
use data_bus::{DataBus, DataPins};
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::{draw_target::DrawTarget, pixelcolor::Rgb565};
use embedded_hal::delay::DelayNs;
use mipidsi::{models::ST7789, Builder};
use stm32f0xx_hal::gpio::{gpioe, gpiof, Floating, Input, Output, PushPull};
use stm32f0xx_hal::prelude::_embedded_hal_gpio_OutputPin;

/// Display dimensions
pub const DISPLAY_WIDTH: u16 = 240;
pub const DISPLAY_HEIGHT: u16 = 320;

pub struct DisplayPoint {
    pub x: u16,
    pub y: u16,
}

pub type DisplaySize = DisplayPoint;

pub type ApusDisplay = mipidsi::Display<DataBus, ST7789, gpiof::PF10<Output<PushPull>>>;

pub struct DisplayPins {
    pub db0: gpioe::PE0<Input<Floating>>,
    pub db1: gpioe::PE1<Input<Floating>>,
    pub db2: gpioe::PE2<Input<Floating>>,
    pub db3: gpioe::PE3<Input<Floating>>,
    pub db4: gpioe::PE4<Input<Floating>>,
    pub db5: gpioe::PE5<Input<Floating>>,
    pub db6: gpioe::PE6<Input<Floating>>,
    pub db7: gpioe::PE7<Input<Floating>>,
    pub n_cs: gpioe::PE8<Input<Floating>>,
    pub d_c: gpioe::PE9<Input<Floating>>,
    pub n_wr: gpioe::PE10<Input<Floating>>,
    pub n_rd: gpioe::PE11<Input<Floating>>,
    pub n_reset: gpiof::PF10<Input<Floating>>,
}

/// Initialize the ST7789 display
pub fn init(pins: DisplayPins, mut delay_source: impl DelayNs) -> ApusDisplay {
    let (data_interface, n_reset) = interrupt::free(|cs| {
        // the data lines, D/C and WR toggle at bus speed, beyond what low-speed outputs drive
        let data = DataPins {
            db0: pins.db0.into_push_pull_output_hs(cs),
            db1: pins.db1.into_push_pull_output_hs(cs),
            db2: pins.db2.into_push_pull_output_hs(cs),
            db3: pins.db3.into_push_pull_output_hs(cs),
            db4: pins.db4.into_push_pull_output_hs(cs),
            db5: pins.db5.into_push_pull_output_hs(cs),
            db6: pins.db6.into_push_pull_output_hs(cs),
            db7: pins.db7.into_push_pull_output_hs(cs),
        };

        let mut n_cs = pins.n_cs.into_push_pull_output(cs);
        let d_c = pins.d_c.into_push_pull_output_hs(cs);
        let mut n_wr = pins.n_wr.into_push_pull_output_hs(cs);
        let mut n_rd = pins.n_rd.into_push_pull_output(cs);
        let n_reset = pins.n_reset.into_push_pull_output(cs);

        // initialise read and write as inactive (negative logic)
        let _ = n_rd.set_high();
        let _ = n_wr.set_high();
        // enable display (negative logic)
        let _ = n_cs.set_low();

        let di = DataBus::new(data, d_c, n_wr);
        (di, n_reset)
    });

    let mut display = Builder::new(ST7789, data_interface)
        .reset_pin(n_reset)
        .display_size(DISPLAY_WIDTH, DISPLAY_HEIGHT)
        .init(&mut delay_source)
        .unwrap();

    // Clear display
    let _ = display.clear(Rgb565::BLACK);

    display
}
