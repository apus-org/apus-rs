use cortex_m::interrupt;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::{draw_target::DrawTarget, pixelcolor::Rgb565};
use embedded_hal::delay::DelayNs;
use mipidsi::interface::{Generic8BitBus, ParallelInterface};
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

pub type ApusDisplay = mipidsi::Display<
    ParallelInterface<
        Generic8BitBus<
            gpioe::PE0<Output<PushPull>>,
            gpioe::PE1<Output<PushPull>>,
            gpioe::PE2<Output<PushPull>>,
            gpioe::PE3<Output<PushPull>>,
            gpioe::PE4<Output<PushPull>>,
            gpioe::PE5<Output<PushPull>>,
            gpioe::PE6<Output<PushPull>>,
            gpioe::PE7<Output<PushPull>>,
        >,
        gpioe::PE9<Output<PushPull>>,
        gpioe::PE10<Output<PushPull>>,
    >,
    ST7789,
    gpiof::PF10<Output<PushPull>>,
>;

pub struct ApusDisplayPins {
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
pub fn display_init(pins: ApusDisplayPins, mut delay_source: impl DelayNs) -> ApusDisplay {
    let (data_interface, n_reset) = interrupt::free(|cs| {
        let db0 = pins.db0.into_push_pull_output(cs);
        let db1 = pins.db1.into_push_pull_output(cs);
        let db2 = pins.db2.into_push_pull_output(cs);
        let db3 = pins.db3.into_push_pull_output(cs);
        let db4 = pins.db4.into_push_pull_output(cs);
        let db5 = pins.db5.into_push_pull_output(cs);
        let db6 = pins.db6.into_push_pull_output(cs);
        let db7 = pins.db7.into_push_pull_output(cs);

        let mut n_cs = pins.n_cs.into_push_pull_output(cs);
        let d_c = pins.d_c.into_push_pull_output(cs);
        let mut n_wr = pins.n_wr.into_push_pull_output(cs);
        let mut n_rd = pins.n_rd.into_push_pull_output(cs);
        let n_reset = pins.n_reset.into_push_pull_output(cs);

        // initialise read and write as inactive (negative logic)
        let _ = n_rd.set_high();
        let _ = n_wr.set_high();
        // enable display (negative logic)
        let _ = n_cs.set_low();

        let bus = Generic8BitBus::new((db0, db1, db2, db3, db4, db5, db6, db7));

        let di = ParallelInterface::new(bus, d_c, n_wr);
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
