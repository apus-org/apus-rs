use stm32f0xx_hal::{
    gpio::{gpioa, gpiob, gpioc, gpiod, gpiof, Floating, Input},
    pac,
    prelude::*,
    rcc::{self, HSEBypassMode, USBClockSource},
};

use crate::{
    bsp::{
        buttons::ButtonPins,
        display::DisplayPins,
        leds::{LedPins, RgbPins},
    },
    peripherals::{
        can::CanPins,
        i2c::i2c1::I2c1Pins,
        serial::{usart3::Usart3Pins, usart4::Usart4Pins},
        spi::spi1::Spi1Pins,
        swd::SwdPins,
        usb::UsbPins,
    },
};

/// The whole board, grouped by what each part is wired to and left unconfigured
pub struct Resources {
    pub rcc: rcc::Rcc,
    pub leds: LedPins,
    pub buttons: ButtonPins,
    pub display: DisplayPins,
    pub spi1: Spi1Pins,
    pub i2c1: I2c1Pins,
    pub can: CanPins,
    pub usart3: Usart3Pins,
    pub usart4: Usart4Pins,
    pub usb: UsbPins,
    pub potentiometer: gpioc::PC4<Input<Floating>>,
    pub buzzer: gpioc::PC9<Input<Floating>>,
    /// multi-function pin of the RTC
    pub rtc_mfp: gpiod::PD3<Input<Floating>>,
    pub header: ArduinoHeader,
    pub swd: SwdPins,
    pub unconnected: UnconnectedPins,
    pub pins: FreePins,
    pub peripherals: Peripherals,
}

/// Peripherals used by the on-board parts, and all others
pub struct Peripherals {
    pub spi1: pac::SPI1,
    pub i2c1: pac::I2C1,
    pub adc: pac::ADC,
    pub can: pac::CAN,
    pub usart3: pac::USART3,
    pub usart4: pac::USART4,
    pub usb: pac::USB,
    pub tim3: pac::TIM3,
    pub free: FreePeripherals,
}

impl Resources {
    /// Sets the system clock to 48 MHz from the 8 MHz crystal and splits the GPIO ports
    pub fn new(peripherals: pac::Peripherals) -> Self {
        let mut p = peripherals;

        // configure clocks: 8 MHz crystal on PF0/PF1, PLL to 48 MHz, which also clocks USB
        let mut rcc = p
            .RCC
            .configure()
            .hse(8.mhz(), HSEBypassMode::NotBypassed)
            .sysclk(48.mhz())
            .usbsrc(USBClockSource::PLL)
            .freeze(&mut p.FLASH);

        let gpioa = p.GPIOA.split(&mut rcc);
        let gpiob = p.GPIOB.split(&mut rcc);
        let gpioc = p.GPIOC.split(&mut rcc);
        let gpiod = p.GPIOD.split(&mut rcc);
        let gpioe = p.GPIOE.split(&mut rcc);
        let gpiof = p.GPIOF.split(&mut rcc);

        let leds = LedPins {
            led3: gpioc.pc0,
            led4: gpioc.pc1,
            led5: gpioc.pc2,
            rgb: RgbPins {
                red: gpioc.pc6,
                green: gpioc.pc7,
                blue: gpioc.pc8,
            },
        };

        let buttons = ButtonPins {
            wake_up: gpioa.pa0,
            user_0: gpiod.pd0,
            user_1: gpiod.pd1,
        };

        let display = DisplayPins {
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

        // touch controller is the only device on SPI1
        let spi1 = Spi1Pins {
            n_cs: gpioe.pe12,
            sck: gpioe.pe13,
            miso: gpioe.pe14,
            mosi: gpioe.pe15,
        };

        // shared by the temperature sensor and RTC
        let i2c1 = I2c1Pins {
            scl: gpiob.pb6,
            sda: gpiob.pb7,
        };

        let can = CanPins {
            rx: gpiob.pb8,
            tx: gpiob.pb9,
            mode: gpioc.pc13,
        };

        // wired to the FTDI USB-UART of the VCOM port
        let usart3 = Usart3Pins {
            tx: gpiod.pd8,
            rx: gpiod.pd9,
        };

        // wired to the UART connector
        let usart4 = Usart4Pins {
            tx: gpioc.pc10,
            rx: gpioc.pc11,
        };

        // USB DATA port
        let usb = UsbPins {
            dm: gpioa.pa11,
            dp: gpioa.pa12,
        };

        // pins routed to the arduino shield header
        let header = ArduinoHeader {
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
        };

        let swd = SwdPins {
            swdio: gpioa.pa13,
            swclk: gpioa.pa14,
        };

        let unconnected = UnconnectedPins {
            pb4: gpiob.pb4,
            pb10: gpiob.pb10,
            pb11: gpiob.pb11,
            pc3: gpioc.pc3,
            pc5: gpioc.pc5,
            pc12: gpioc.pc12,
            pc14: gpioc.pc14,
            pc15: gpioc.pc15,
            pd2: gpiod.pd2,
            pd5: gpiod.pd5,
            pd6: gpiod.pd6,
            pd7: gpiod.pd7,
            pd10: gpiod.pd10,
            pd13: gpiod.pd13,
            pd14: gpiod.pd14,
            pd15: gpiod.pd15,
            pf6: gpiof.pf6,
            pf9: gpiof.pf9,
        };

        // pins wired on the board that have no place yet
        let free_pins = FreePins {
            pd4: gpiod.pd4,
            pd11: gpiod.pd11,
            pd12: gpiod.pd12,
            pf2: gpiof.pf2,
            pf3: gpiof.pf3,
        };

        // board-peripherals that are unused
        let free_peripherals = FreePeripherals {
            crc: p.CRC,
            spi2: p.SPI2,
            dac: p.DAC,
            pwr: p.PWR,
            i2c2: p.I2C2,
            iwdg: p.IWDG,
            wwdg: p.WWDG,
            tim1: p.TIM1,
            tim2: p.TIM2,
            tim14: p.TIM14,
            tim6: p.TIM6,
            tim7: p.TIM7,
            exti: p.EXTI,
            dma1: p.DMA1,
            syscfg: p.SYSCFG,
            usart1: p.USART1,
            usart2: p.USART2,
            rtc: p.RTC,
            tim15: p.TIM15,
            tim16: p.TIM16,
            tim17: p.TIM17,
            tsc: p.TSC,
            cec: p.CEC,
            flash: p.FLASH,
            dbgmcu: p.DBGMCU,
            crs: p.CRS,
            stk: p.STK,
            comp: p.COMP,
        };

        Self {
            leds,
            buttons,
            display,
            spi1,
            i2c1,
            can,
            usart3,
            usart4,
            usb,
            potentiometer: gpioc.pc4,
            buzzer: gpioc.pc9,
            rtc_mfp: gpiod.pd3,
            header,
            swd,
            unconnected,
            pins: free_pins,
            peripherals: Peripherals {
                spi1: p.SPI1,
                i2c1: p.I2C1,
                adc: p.ADC,
                can: p.CAN,
                usart3: p.USART3,
                usart4: p.USART4,
                usb: p.USB,
                tim3: p.TIM3,
                free: free_peripherals,
            },
            rcc,
        }
    }
}

pub struct FreePeripherals {
    pub crc: pac::CRC,
    pub spi2: pac::SPI2,
    pub dac: pac::DAC,
    pub pwr: pac::PWR,
    pub i2c2: pac::I2C2,
    pub iwdg: pac::IWDG,
    pub wwdg: pac::WWDG,
    pub tim1: pac::TIM1,
    pub tim2: pac::TIM2,
    pub tim14: pac::TIM14,
    pub tim6: pac::TIM6,
    pub tim7: pac::TIM7,
    pub exti: pac::EXTI,
    pub dma1: pac::DMA1,
    pub syscfg: pac::SYSCFG,
    pub usart1: pac::USART1,
    pub usart2: pac::USART2,
    pub rtc: pac::RTC,
    pub tim15: pac::TIM15,
    pub tim16: pac::TIM16,
    pub tim17: pac::TIM17,
    pub tsc: pac::TSC,
    pub cec: pac::CEC,
    pub flash: pac::FLASH,
    pub dbgmcu: pac::DBGMCU,
    pub crs: pac::CRS,
    pub stk: pac::STK,
    pub comp: pac::COMP,
}

/// Pins routed to the Arduino Uno R3 shield header, left unconfigured.
pub struct ArduinoHeader {
    /// PA10 · USART1_RX
    pub d0: gpioa::PA10<Input<Floating>>,
    /// PA9 · USART1_TX
    pub d1: gpioa::PA9<Input<Floating>>,
    /// PB5 · INT0
    pub d2: gpiob::PB5<Input<Floating>>,
    /// PB2 · INT1
    pub d3: gpiob::PB2<Input<Floating>>,
    /// PA8 · TIM1_CH1
    pub d4: gpioa::PA8<Input<Floating>>,
    /// PA7 · TIM3_CH2
    pub d5: gpioa::PA7<Input<Floating>>,
    /// PA3 · COMP2_INP
    pub d6: gpioa::PA3<Input<Floating>>,
    /// PA5 · COMP2_INM
    pub d7: gpioa::PA5<Input<Floating>>,
    /// PA15 · TIM2_CH1
    pub d8: gpioa::PA15<Input<Floating>>,
    /// PB3 · TIM2_CH2
    pub d9: gpiob::PB3<Input<Floating>>,
    /// PB12 · SPI2_NSS
    pub d10: gpiob::PB12<Input<Floating>>,
    /// PB15 · SPI2_MOSI
    pub d11: gpiob::PB15<Input<Floating>>,
    /// PB14 · SPI2_MISO
    pub d12: gpiob::PB14<Input<Floating>>,
    /// PB13 · SPI2_SCK
    pub d13: gpiob::PB13<Input<Floating>>,
    /// PA1 · ADC_IN1
    pub a0: gpioa::PA1<Input<Floating>>,
    /// PA2 · ADC_IN2
    pub a1: gpioa::PA2<Input<Floating>>,
    /// PA4 · ADC_IN4
    pub a2: gpioa::PA4<Input<Floating>>,
    /// PA6 · ADC_IN6
    pub a3: gpioa::PA6<Input<Floating>>,
    /// PB0 · ADC_IN8, via SJ2 (header pin A4/SDA)
    pub a4: gpiob::PB0<Input<Floating>>,
    /// PB1 · ADC_IN9, via SJ1 (header pin A5/SCL)
    pub a5: gpiob::PB1<Input<Floating>>,
}

/// Pins that are not connected on the board
pub struct UnconnectedPins {
    pub pb4: gpiob::PB4<Input<Floating>>,
    /// I2C2_SCL; reaches the header's A5/SCL only with SJ1 moved
    pub pb10: gpiob::PB10<Input<Floating>>,
    /// I2C2_SDA; reaches the header's A4/SDA only with SJ2 moved
    pub pb11: gpiob::PB11<Input<Floating>>,
    pub pc3: gpioc::PC3<Input<Floating>>,
    pub pc5: gpioc::PC5<Input<Floating>>,
    pub pc12: gpioc::PC12<Input<Floating>>,
    pub pc14: gpioc::PC14<Input<Floating>>,
    pub pc15: gpioc::PC15<Input<Floating>>,
    pub pd2: gpiod::PD2<Input<Floating>>,
    pub pd5: gpiod::PD5<Input<Floating>>,
    pub pd6: gpiod::PD6<Input<Floating>>,
    pub pd7: gpiod::PD7<Input<Floating>>,
    pub pd10: gpiod::PD10<Input<Floating>>,
    pub pd13: gpiod::PD13<Input<Floating>>,
    pub pd14: gpiod::PD14<Input<Floating>>,
    pub pd15: gpiod::PD15<Input<Floating>>,
    pub pf6: gpiof::PF6<Input<Floating>>,
    pub pf9: gpiof::PF9<Input<Floating>>,
}

/// Pins wired on the board that have no place yet
pub struct FreePins {
    /// PD4 · temperature sensor alert
    pub pd4: gpiod::PD4<Input<Floating>>,
    /// PD11 · VCOM CTS
    pub pd11: gpiod::PD11<Input<Floating>>,
    /// PD12 · VCOM RTS
    pub pd12: gpiod::PD12<Input<Floating>>,
    /// PF2 · touch pen interrupt
    pub pf2: gpiof::PF2<Input<Floating>>,
    /// PF3 · touch busy
    pub pf3: gpiof::PF3<Input<Floating>>,
}
