use crate::bsp::{
    buttons::Buttons,
    delay::BusyDelay,
    display::{display_init, ApusDisplay, ApusDisplayPins},
    leds::Leds,
    potentiometer::Potentiometer,
};

use stm32f0xx_hal::{
    adc::Adc,
    gpio::{gpioa, gpiob, gpioc, gpiod, gpioe, gpiof, Floating, Input},
    pac,
    prelude::*,
    rcc,
};

macro_rules! optional_struct {
    (
        $struct_name:ident {
            $($field_name:ident : $field_ty:ty),* $(,)?
        }
    ) => {
        pub struct $struct_name {
            $(pub $field_name: Option<$field_ty>,)*
        }

        impl $struct_name {
            /// Take a member out, leaving None behind
            pub fn take_field<T>(&mut self, field: &mut Option<T>) -> T {
                field.take().expect("field already taken")
            }
        }
    };
}

macro_rules! optional_struct_init {
    (
        $struct_name:ident {
            $($field_name:ident : $expr:expr),* $(,)?
        }
    ) => {
        $struct_name {
            $($field_name: Some($expr),)*
        }
    };
}

pub struct Board {
    pub leds: Leds,
    pub buttons: Buttons,
    pub display: ApusDisplay,
    pub potentiometer: Potentiometer,

    pub delay: BusyDelay,
    pub peripherals: BoardPeripherals,
    pub pins: FreePins,
}

impl Board {
    pub fn new(peripherals: pac::Peripherals) -> Self {
        let mut p = peripherals;

        // configure clocks
        let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);
        // split GPIOA and GPIOD for the buttons
        let gpioa = p.GPIOA.split(&mut rcc);
        let gpiod = p.GPIOD.split(&mut rcc);
        // split GPIOC for the LEDs
        let gpioc = p.GPIOC.split(&mut rcc);
        // split GPIOE/F for the display
        let gpioe = p.GPIOE.split(&mut rcc);
        let gpiof = p.GPIOF.split(&mut rcc);
        // split GPIOB for leftover pins
        let gpiob = p.GPIOB.split(&mut rcc);

        // delay for display initialisation
        let sys_clock_hz = rcc.clocks.sysclk().0;
        let delay_display = BusyDelay::new(sys_clock_hz);

        // display init
        let display_pins = ApusDisplayPins {
            db0: gpioe.pe0,
            db1: gpioe.pe1,
            db2: gpioe.pe2,
            db3: gpioe.pe3,
            db4: gpioe.pe4,
            db5: gpioe.pe5,
            db6: gpioe.pe6,
            db7: gpioe.pe7,
            n_cs: gpioe.pe8,
            d_c: gpioe.pe9,
            n_wr: gpioe.pe10,
            n_rd: gpioe.pe11,
            n_reset: gpiof.pf10,
        };

        let display = display_init(display_pins, delay_display);

        // led construction
        let leds = Leds::new(
            gpioc.pc0, gpioc.pc1, gpioc.pc2, gpioc.pc6, gpioc.pc7, gpioc.pc8,
        );

        // button construction
        let buttons = Buttons::new(gpioa.pa0, gpiod.pd0, gpiod.pd1);

        // poti construction
        let adc = Adc::new(p.ADC, &mut rcc);
        let potentiometer = Potentiometer::new(gpioc.pc4, adc);

        // user delay
        let delay = BusyDelay::new(sys_clock_hz);

        // board-peripherals that were configured/modified during initialisation
        let configured_peripherals = ConfiguredPeripherals { rcc };

        // leftover pins from splitting gpios
        let free_pins = optional_struct_init!(FreePins {
            // GPIOA leftovers
            pa1: gpioa.pa1,
            pa2: gpioa.pa2,
            pa3: gpioa.pa3,
            pa4: gpioa.pa4,
            pa5: gpioa.pa5,
            pa6: gpioa.pa6,
            pa7: gpioa.pa7,
            pa8: gpioa.pa8,
            pa9: gpioa.pa9,
            pa10: gpioa.pa10,
            pa11: gpioa.pa11,
            pa12: gpioa.pa12,
            pa13: gpioa.pa13,
            pa14: gpioa.pa14,
            pa15: gpioa.pa15,

            // GPIOB
            pb0: gpiob.pb0,
            pb1: gpiob.pb1,
            pb2: gpiob.pb2,
            pb3: gpiob.pb3,
            pb4: gpiob.pb4,
            pb5: gpiob.pb5,
            pb6: gpiob.pb6,
            pb7: gpiob.pb7,
            pb8: gpiob.pb8,
            pb9: gpiob.pb9,
            pb10: gpiob.pb10,
            pb11: gpiob.pb11,
            pb12: gpiob.pb12,
            pb13: gpiob.pb13,
            pb14: gpiob.pb14,
            pb15: gpiob.pb15,

            // GPIOC leftovers
            pc3: gpioc.pc3,
            pc5: gpioc.pc5,
            pc9: gpioc.pc9,
            pc10: gpioc.pc10,
            pc11: gpioc.pc11,
            pc12: gpioc.pc12,
            pc13: gpioc.pc13,
            pc14: gpioc.pc14,
            pc15: gpioc.pc15,

            // GPIOD leftovers
            pd2: gpiod.pd2,
            pd3: gpiod.pd3,
            pd4: gpiod.pd4,
            pd5: gpiod.pd5,
            pd6: gpiod.pd6,
            pd7: gpiod.pd7,
            pd8: gpiod.pd8,
            pd9: gpiod.pd9,
            pd10: gpiod.pd10,
            pd11: gpiod.pd11,
            pd12: gpiod.pd12,
            pd13: gpiod.pd13,
            pd14: gpiod.pd14,
            pd15: gpiod.pd15,

            // GPIOE leftovers
            pe12: gpioe.pe12,
            pe13: gpioe.pe13,
            pe14: gpioe.pe14,
            pe15: gpioe.pe15,

            // GPIOF leftovers
            pf0: gpiof.pf0,
            pf1: gpiof.pf1,
            pf2: gpiof.pf2,
            pf3: gpiof.pf3,
            pf6: gpiof.pf6,
            pf9: gpiof.pf9,
        });

        // board-peripherals that are unused
        let free_peripherals = optional_struct_init!(FreePeripherals {
            crc: p.CRC,
            spi1: p.SPI1,
            spi2: p.SPI2,
            dac: p.DAC,
            pwr: p.PWR,
            i2c1: p.I2C1,
            i2c2: p.I2C2,
            iwdg: p.IWDG,
            wwdg: p.WWDG,
            tim1: p.TIM1,
            tim2: p.TIM2,
            tim3: p.TIM3,
            tim14: p.TIM14,
            tim6: p.TIM6,
            tim7: p.TIM7,
            exti: p.EXTI,
            dma1: p.DMA1,
            syscfg: p.SYSCFG,
            usart1: p.USART1,
            usart2: p.USART2,
            usart3: p.USART3,
            usart4: p.USART4,
            rtc: p.RTC,
            tim15: p.TIM15,
            tim16: p.TIM16,
            tim17: p.TIM17,
            tsc: p.TSC,
            cec: p.CEC,
            flash: p.FLASH,
            dbgmcu: p.DBGMCU,
            usb: p.USB,
            crs: p.CRS,
            can: p.CAN,
            stk: p.STK,
            comp: p.COMP,
        });

        let peripherals = BoardPeripherals {
            configured: configured_peripherals,
            free: free_peripherals,
        };

        Self {
            leds,
            buttons,
            display,
            potentiometer,
            delay,

            peripherals,
            pins: free_pins,
        }
    }
}

pub struct BoardPeripherals {
    pub configured: ConfiguredPeripherals,
    pub free: FreePeripherals,
}

pub struct ConfiguredPeripherals {
    pub rcc: rcc::Rcc,
}

optional_struct!(FreePeripherals {
    crc: pac::CRC,
    spi1: pac::SPI1,
    spi2: pac::SPI2,
    dac: pac::DAC,
    pwr: pac::PWR,
    i2c1: pac::I2C1,
    i2c2: pac::I2C2,
    iwdg: pac::IWDG,
    wwdg: pac::WWDG,
    tim1: pac::TIM1,
    tim2: pac::TIM2,
    tim3: pac::TIM3,
    tim14: pac::TIM14,
    tim6: pac::TIM6,
    tim7: pac::TIM7,
    exti: pac::EXTI,
    dma1: pac::DMA1,
    syscfg: pac::SYSCFG,
    usart1: pac::USART1,
    usart2: pac::USART2,
    usart3: pac::USART3,
    usart4: pac::USART4,
    rtc: pac::RTC,
    tim15: pac::TIM15,
    tim16: pac::TIM16,
    tim17: pac::TIM17,
    tsc: pac::TSC,
    cec: pac::CEC,
    flash: pac::FLASH,
    dbgmcu: pac::DBGMCU,
    usb: pac::USB,
    crs: pac::CRS,
    can: pac::CAN,
    stk: pac::STK,
    comp: pac::COMP,
});

optional_struct!(FreePins {
    // GPIOA leftovers
    pa1: gpioa::PA1<Input<Floating>>,
    pa2: gpioa::PA2<Input<Floating>>,
    pa3: gpioa::PA3<Input<Floating>>,
    pa4: gpioa::PA4<Input<Floating>>,
    pa5: gpioa::PA5<Input<Floating>>,
    pa6: gpioa::PA6<Input<Floating>>,
    pa7: gpioa::PA7<Input<Floating>>,
    pa8: gpioa::PA8<Input<Floating>>,
    pa9: gpioa::PA9<Input<Floating>>,
    pa10: gpioa::PA10<Input<Floating>>,
    pa11: gpioa::PA11<Input<Floating>>,
    pa12: gpioa::PA12<Input<Floating>>,
    pa13: gpioa::PA13<Input<Floating>>,
    pa14: gpioa::PA14<Input<Floating>>,
    pa15: gpioa::PA15<Input<Floating>>,
    // GPIOB leftovers
    pb0: gpiob::PB0<Input<Floating>>,
    pb1: gpiob::PB1<Input<Floating>>,
    pb2: gpiob::PB2<Input<Floating>>,
    pb3: gpiob::PB3<Input<Floating>>,
    pb4: gpiob::PB4<Input<Floating>>,
    pb5: gpiob::PB5<Input<Floating>>,
    pb6: gpiob::PB6<Input<Floating>>,
    pb7: gpiob::PB7<Input<Floating>>,
    pb8: gpiob::PB8<Input<Floating>>,
    pb9: gpiob::PB9<Input<Floating>>,
    pb10: gpiob::PB10<Input<Floating>>,
    pb11: gpiob::PB11<Input<Floating>>,
    pb12: gpiob::PB12<Input<Floating>>,
    pb13: gpiob::PB13<Input<Floating>>,
    pb14: gpiob::PB14<Input<Floating>>,
    pb15: gpiob::PB15<Input<Floating>>,

    // GPIOC leftovers
    pc3: gpioc::PC3<Input<Floating>>,
    pc5: gpioc::PC5<Input<Floating>>,
    pc9: gpioc::PC9<Input<Floating>>,
    pc10: gpioc::PC10<Input<Floating>>,
    pc11: gpioc::PC11<Input<Floating>>,
    pc12: gpioc::PC12<Input<Floating>>,
    pc13: gpioc::PC13<Input<Floating>>,
    pc14: gpioc::PC14<Input<Floating>>,
    pc15: gpioc::PC15<Input<Floating>>,

    // GPIOD leftovers
    pd2: gpiod::PD2<Input<Floating>>,
    pd3: gpiod::PD3<Input<Floating>>,
    pd4: gpiod::PD4<Input<Floating>>,
    pd5: gpiod::PD5<Input<Floating>>,
    pd6: gpiod::PD6<Input<Floating>>,
    pd7: gpiod::PD7<Input<Floating>>,
    pd8: gpiod::PD8<Input<Floating>>,
    pd9: gpiod::PD9<Input<Floating>>,
    pd10: gpiod::PD10<Input<Floating>>,
    pd11: gpiod::PD11<Input<Floating>>,
    pd12: gpiod::PD12<Input<Floating>>,
    pd13: gpiod::PD13<Input<Floating>>,
    pd14: gpiod::PD14<Input<Floating>>,
    pd15: gpiod::PD15<Input<Floating>>,

    // GPIOE leftovers
    pe12: gpioe::PE12<Input<Floating>>,
    pe13: gpioe::PE13<Input<Floating>>,
    pe14: gpioe::PE14<Input<Floating>>,
    pe15: gpioe::PE15<Input<Floating>>,

    // GPIOF leftovers
    pf0: gpiof::PF0<Input<Floating>>,
    pf1: gpiof::PF1<Input<Floating>>,
    pf2: gpiof::PF2<Input<Floating>>,
    pf3: gpiof::PF3<Input<Floating>>,
    pf6: gpiof::PF6<Input<Floating>>,
    pf9: gpiof::PF9<Input<Floating>>,
});
