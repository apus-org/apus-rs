use mipidsi::interface::{Generic8BitBus, ParallelInterface};
use mipidsi::Display;

use mipidsi::{models::ST7789, Builder};

use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

use embedded_hal_compat::ForwardCompat;

use cortex_m::interrupt;
use embedded_hal::digital::v2::OutputPin;
use stm32f0xx_hal::pac;

// Display
const H: i32 = 320;
const W: i32 = 240;

pub fn make_display(
    gpioe: stm32f0xx_hal::gpio::gpioe::Parts,
    gpiof: stm32f0xx_hal::gpio::gpiof::Parts,
    core: pac::CorePeripherals,
    rcc: &mut stm32f0xx_hal::rcc::Rcc,
) -> impl embedded_graphics::prelude::DrawTarget<Color = Rgb565> {
    interrupt::free(|cs| {
        let db0 = gpioe.pe0.into_push_pull_output(cs);
        let db1 = gpioe.pe1.into_push_pull_output(cs);
        let db2 = gpioe.pe2.into_push_pull_output(cs);
        let db3 = gpioe.pe3.into_push_pull_output(cs);
        let db4 = gpioe.pe4.into_push_pull_output(cs);
        let db5 = gpioe.pe5.into_push_pull_output(cs);
        let db6 = gpioe.pe6.into_push_pull_output(cs);
        let db7 = gpioe.pe7.into_push_pull_output(cs);

        let dc = gpioe.pe9.into_push_pull_output(cs);
        let wr = gpioe.pe10.into_push_pull_output(cs);
        let rst = gpiof.pf10.into_push_pull_output(cs);

        let bus = Generic8BitBus::new((
            db0.forward(),
            db1.forward(),
            db2.forward(),
            db3.forward(),
            db4.forward(),
            db5.forward(),
            db6.forward(),
            db7.forward(),
        ));

        let di = ParallelInterface::new(bus, dc.forward(), wr.forward());
        let mut delay = cortex_m::delay::Delay::new(core.SYST, rcc.clocks.sysclk().0);

        Builder::new(ST7789, di)
            .reset_pin(rst.forward())
            .color_order(mipidsi::options::ColorOrder::Rgb)
            .display_size(W as u16, H as u16)
            // .orientation(orientation)
            .init(&mut delay.forward())
            .unwrap()
    })
}
