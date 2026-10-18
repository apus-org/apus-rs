pub mod resources;

use resources::{ArduinoHeader, FreePeripherals, FreePins, Resources};

use crate::{
    bsp::{
        buttons::Buttons,
        buzzer::Buzzer,
        delay::BusyDelay,
        display::{self, ApusDisplay, DisplaySize, DISPLAY_HEIGHT, DISPLAY_WIDTH},
        leds::Leds,
        potentiometer::Potentiometer,
        rtc::{self, ApusRtc},
        temperature::{self, ApusTemperature},
        touch::{self, Touch},
    },
    peripherals::{
        can::{self, CanBus},
        i2c::i2c1::{self, I2c1},
        serial::usart3::{self, Usart3},
        spi::spi1,
    },
};

use core::cell::RefCell;

use cortex_m::interrupt;
use cortex_m::singleton;

use critical_section::Mutex;
use embedded_hal_bus::i2c::CriticalSectionDevice;
use embedded_hal_bus::spi::ExclusiveDevice;

use stm32f0xx_hal::{adc::Adc, pac, rcc, time::Bps};

type BoardI2c = CriticalSectionDevice<'static, I2c1>;

/// Baud rate of [`Board::vcom`]
pub const VCOM_BAUD_RATE: u32 = 115_200;

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
    pub can: CanBus,
    pub vcom: Usart3,

    pub header: ArduinoHeader,
    pub peripherals: BoardPeripherals,
    pub pins: FreePins,
}

impl Board {
    pub fn new(peripherals: pac::Peripherals) -> Self {
        let res = Resources::new(peripherals);
        let mut rcc = res.rcc;
        let sys_clock_hz = rcc.clocks.sysclk().0;

        // display construction
        let display = display::init(res.display, BusyDelay::new(sys_clock_hz));

        // spi1 construction (touch controller is the only device on SPI1)
        let (spi1_bus, n_cs) = spi1::init(res.peripherals.spi1, res.spi1, &mut rcc);

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
        let i2c1_bus = i2c1::init(res.peripherals.i2c1, res.i2c1, &mut rcc);
        let i2c1_bus_static = Self::install_i2c1_singleton(i2c1_bus);
        let temperature = temperature::new(CriticalSectionDevice::new(i2c1_bus_static));
        let rtc = rtc::new(CriticalSectionDevice::new(i2c1_bus_static));

        // can construction
        let (can_instance, can_mode) = can::init(res.peripherals.can, res.can, &mut rcc);
        let can = CanBus::new(can_instance, can_mode);

        // vcom construction (USART3 to the FTDI USB-UART)
        let vcom = usart3::init(
            res.peripherals.usart3,
            res.usart3,
            Bps(VCOM_BAUD_RATE),
            &mut rcc,
        );

        // led construction
        let leds = Leds::new(res.leds);

        // button construction
        let buttons = Buttons::new(res.buttons);

        // poti construction
        let adc = Adc::new(res.peripherals.adc, &mut rcc);
        let potentiometer = Potentiometer::new(res.potentiometer, adc);

        // buzzer construction
        let buzzer = Buzzer::new(res.buzzer);

        // user delay
        let delay = BusyDelay::new(sys_clock_hz);

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
            can,
            vcom,

            header: res.header,
            peripherals: BoardPeripherals {
                configured: ConfiguredPeripherals { rcc },
                free: res.peripherals.free,
            },
            pins: res.pins,
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
