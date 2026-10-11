use crate::{
    bsp::{
        buttons::Buttons,
        buzzer::Buzzer,
        delay::BusyDelay,
        display::{self, ApusDisplay, ApusDisplayPins, DisplaySize, DISPLAY_HEIGHT, DISPLAY_WIDTH},
        leds::Leds,
        potentiometer::Potentiometer,
        rtc::{self, ApusRtc},
        temperature::{self, ApusTemperature},
        touch::{self, Touch},
    },
    peripherals::{
        i2c::i2c1::{self, I2c1},
        spi::spi1,
    },
};

use core::cell::RefCell;

use cortex_m::interrupt;
use cortex_m::singleton;

use critical_section::Mutex;
use embedded_hal_bus::i2c::CriticalSectionDevice;
use embedded_hal_bus::spi::ExclusiveDevice;

use stm32f0xx_hal::{
    adc::Adc,
    gpio::{gpioa, gpiob, gpioc, gpiod, gpiof, Floating, Input},
    pac,
    prelude::*,
    rcc,
};

type BoardI2c = CriticalSectionDevice<'static, I2c1>;

pub struct Board {
    pub leds: Leds,
    pub buttons: Buttons,
    pub display: ApusDisplay,
    pub potentiometer: Potentiometer,
    pub buzzer: Buzzer,
    pub delay: BusyDelay,
    pub touch: Touch,
    pub temperature: ApusTemperature<BoardI2c>,
    pub rtc: ApusRtc<BoardI2c>,

    pub header: ArduinoHeader,
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

        // display construction
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

        let display = display::init(display_pins, delay_display);

        // spi1 construction (touch controller is the only device on SPI1)
        let (spi1_bus, n_cs) = spi1::init(
            p.SPI1, gpioe.pe12, gpioe.pe13, gpioe.pe14, gpioe.pe15, &mut rcc,
        );

        // touch driver owns the SPI1 bus exclusively
        let spi1_cs_delay = BusyDelay::new(sys_clock_hz);
        let touch_device = ExclusiveDevice::new(spi1_bus, n_cs, spi1_cs_delay).unwrap();
        let touch = Touch::new(
            touch_device,
            touch::DEFAULT_THRESHOLD,
            DisplaySize {
                x: DISPLAY_WIDTH,
                y: DISPLAY_HEIGHT,
            },
        );

        // i2c1 construction (shared by the temperature sensor and RTC)
        let i2c1_bus = i2c1::init(p.I2C1, gpiob.pb6, gpiob.pb7, &mut rcc);
        let i2c1_bus_static = Self::install_i2c1_singleton(i2c1_bus);
        let temperature = temperature::new(CriticalSectionDevice::new(i2c1_bus_static));
        let rtc = rtc::new(CriticalSectionDevice::new(i2c1_bus_static));

        // led construction
        let leds = Leds::new(
            gpioc.pc0, gpioc.pc1, gpioc.pc2, gpioc.pc6, gpioc.pc7, gpioc.pc8,
        );

        // button construction
        let buttons = Buttons::new(gpioa.pa0, gpiod.pd0, gpiod.pd1);

        // poti construction
        let adc = Adc::new(p.ADC, &mut rcc);
        let potentiometer = Potentiometer::new(gpioc.pc4, adc);

        // buzzer construction
        let buzzer = Buzzer::new(gpioc.pc9);

        // user delay
        let delay = BusyDelay::new(sys_clock_hz);

        // pins routed to the arduino shield header
        let header = optional_struct_init!(ArduinoHeader {
            d0: gpioa.pa10,
            d1: gpioa.pa9,
            d2: gpiob.pb5,
            d3: gpiob.pb2,
            d4: gpioa.pa8,
            d5: gpioa.pa7,
            d6: gpioa.pa3,
            d7: gpioa.pa5,
            d8: gpioa.pa15,
            d9: gpiob.pb3,
            d10: gpiob.pb12,
            d11: gpiob.pb15,
            d12: gpiob.pb14,
            d13: gpiob.pb13,
            a0: gpioa.pa1,
            a1: gpioa.pa2,
            a2: gpioa.pa4,
            a3: gpioa.pa6,
            a4: gpiob.pb0,
            a5: gpiob.pb1,
        });

        // leftover pins from splitting gpios
        let free_pins = optional_struct_init!(FreePins {
            // GPIOA leftovers
            pa11: gpioa.pa11,
            pa12: gpioa.pa12,
            pa13: gpioa.pa13,
            pa14: gpioa.pa14,

            // GPIOB
            pb4: gpiob.pb4,
            pb8: gpiob.pb8,
            pb9: gpiob.pb9,
            pb10: gpiob.pb10,
            pb11: gpiob.pb11,

            // GPIOC leftovers
            pc3: gpioc.pc3,
            pc5: gpioc.pc5,
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
            spi2: p.SPI2,
            dac: p.DAC,
            pwr: p.PWR,
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

        // board-peripherals that were configured/modified during initialisation
        let configured_peripherals = ConfiguredPeripherals { rcc };

        let peripherals = BoardPeripherals {
            configured: configured_peripherals,
            free: free_peripherals,
        };

        Self {
            leds,
            buttons,
            display,
            potentiometer,
            buzzer,
            delay,
            touch,
            temperature,
            rtc,

            header,
            peripherals,
            pins: free_pins,
        }
    }

    /// Install the I2C1 bus into a static singleton
    fn install_i2c1_singleton(bus: I2c1) -> &'static Mutex<RefCell<I2c1>> {
        let bus_static: &'static Mutex<RefCell<I2c1>> = interrupt::free(|_| {
            let slot = singleton!(: Mutex<RefCell<I2c1>> = Mutex::new(RefCell::new(bus)))
                .expect("failed to install i2c1 singleton");
            &*slot
        });

        bus_static
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
    spi2: pac::SPI2,
    dac: pac::DAC,
    pwr: pac::PWR,
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

optional_struct!(
    /// Pins routed to the Arduino Uno R3 shield header, left unconfigured.
    ///
    /// A4 and A5 reach PB0 and PB1 through the 0 Ω jumpers SJ2 and SJ1. Moving those
    /// jumpers routes the header to PB11/PB10 (I2C2) instead, which are then taken from
    /// [`FreePins`]. The SDA/SCL pull-ups R50/R51 are not fitted.
    ArduinoHeader {
        /// PA10 · USART1_RX
        d0: gpioa::PA10<Input<Floating>>,
        /// PA9 · USART1_TX
        d1: gpioa::PA9<Input<Floating>>,
        /// PB5 · INT0
        d2: gpiob::PB5<Input<Floating>>,
        /// PB2 · INT1
        d3: gpiob::PB2<Input<Floating>>,
        /// PA8 · TIM1_CH1
        d4: gpioa::PA8<Input<Floating>>,
        /// PA7 · TIM3_CH2
        d5: gpioa::PA7<Input<Floating>>,
        /// PA3 · COMP2_INP
        d6: gpioa::PA3<Input<Floating>>,
        /// PA5 · COMP2_INM
        d7: gpioa::PA5<Input<Floating>>,
        /// PA15 · TIM2_CH1
        d8: gpioa::PA15<Input<Floating>>,
        /// PB3 · TIM2_CH2
        d9: gpiob::PB3<Input<Floating>>,
        /// PB12 · SPI2_NSS
        d10: gpiob::PB12<Input<Floating>>,
        /// PB15 · SPI2_MOSI
        d11: gpiob::PB15<Input<Floating>>,
        /// PB14 · SPI2_MISO
        d12: gpiob::PB14<Input<Floating>>,
        /// PB13 · SPI2_SCK
        d13: gpiob::PB13<Input<Floating>>,
        /// PA1 · ADC_IN1
        a0: gpioa::PA1<Input<Floating>>,
        /// PA2 · ADC_IN2
        a1: gpioa::PA2<Input<Floating>>,
        /// PA4 · ADC_IN4
        a2: gpioa::PA4<Input<Floating>>,
        /// PA6 · ADC_IN6
        a3: gpioa::PA6<Input<Floating>>,
        /// PB0 · ADC_IN8, via SJ2
        a4: gpiob::PB0<Input<Floating>>,
        /// PB1 · ADC_IN9, via SJ1
        a5: gpiob::PB1<Input<Floating>>,
    }
);

optional_struct!(FreePins {
    // GPIOA leftovers
    pa11: gpioa::PA11<Input<Floating>>,
    pa12: gpioa::PA12<Input<Floating>>,
    pa13: gpioa::PA13<Input<Floating>>,
    pa14: gpioa::PA14<Input<Floating>>,
    // GPIOB leftovers
    pb4: gpiob::PB4<Input<Floating>>,
    pb8: gpiob::PB8<Input<Floating>>,
    pb9: gpiob::PB9<Input<Floating>>,
    pb10: gpiob::PB10<Input<Floating>>,
    pb11: gpiob::PB11<Input<Floating>>,

    // GPIOC leftovers
    pc3: gpioc::PC3<Input<Floating>>,
    pc5: gpioc::PC5<Input<Floating>>,
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

    // GPIOF leftovers
    pf0: gpiof::PF0<Input<Floating>>,
    pf1: gpiof::PF1<Input<Floating>>,
    pf2: gpiof::PF2<Input<Floating>>,
    pf3: gpiof::PF3<Input<Floating>>,
    pf6: gpiof::PF6<Input<Floating>>,
    pf9: gpiof::PF9<Input<Floating>>,
});
