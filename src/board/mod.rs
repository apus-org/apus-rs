pub mod defaults;
pub mod resources;

use defaults::{I2C1_FREQUENCY, RGB_PWM_FREQUENCY, SPI1_FREQUENCY, UART_BAUD_RATE, VCOM_BAUD_RATE};
use resources::{ArduinoHeader, FreePeripherals, Resources, UnconnectedPins};

use crate::{
    bsp::{
        buttons::Buttons,
        buzzer::Buzzer,
        delay::BusyDelay,
        display::{self, ApusDisplay, DisplaySize, DISPLAY_HEIGHT, DISPLAY_WIDTH},
        leds::Leds,
        potentiometer::Potentiometer,
        rtc::Rtc,
        temperature::Temperature,
        touch::{self, Touch},
        vcom::Vcom,
    },
    peripherals::{
        can::{self, CanBus},
        i2c::i2c1::{self, I2c1},
        serial::{
            usart3,
            usart4::{self, Usart4},
        },
        spi::spi1,
        swd::{self, Swd},
        usb::{self, ApusUsbBus},
    },
};

use core::cell::RefCell;

use cortex_m::interrupt;
use cortex_m::singleton;

use critical_section::Mutex;
use embedded_hal_bus::i2c::CriticalSectionDevice;
use embedded_hal_bus::spi::ExclusiveDevice;

use stm32f0xx_hal::{adc::Adc, gpio::Analog, pac, rcc};

type BoardI2c = CriticalSectionDevice<'static, I2c1>;

pub struct Board {
    pub leds: Leds,
    pub buttons: Buttons,
    pub display: ApusDisplay,
    pub adc: Adc,
    pub potentiometer: Potentiometer,
    pub buzzer: Buzzer,
    pub delay: BusyDelay,
    pub touch: Touch,
    pub temperature: Temperature<BoardI2c>,
    pub rtc: Rtc<BoardI2c>,

    pub can: CanBus,
    pub vcom: Vcom,
    pub uart: Usart4,
    pub usb: &'static ApusUsbBus,
    pub swd: Swd,

    pub pins: BoardPins,
    pub peripherals: BoardPeripherals,
}

impl Board {
    pub fn new(peripherals: pac::Peripherals) -> Self {
        let res = Resources::new(peripherals);
        let mut rcc = res.rcc;
        let sys_clock_hz = rcc.clocks.sysclk().0;

        // display construction
        let display = display::init(res.display, BusyDelay::new(sys_clock_hz));

        // spi1 construction (touch controller is the only device on SPI1)
        let (spi1_bus, n_cs) = spi1::init(res.peripherals.spi1, res.spi1, SPI1_FREQUENCY, &mut rcc);

        // touch driver owns the SPI1 bus exclusively
        let spi1_cs_delay = BusyDelay::new(sys_clock_hz);
        let touch_device = ExclusiveDevice::new(spi1_bus, n_cs, spi1_cs_delay).unwrap();
        let touch = Touch::new(
            touch_device,
            res.touch,
            touch::DEFAULT_THRESHOLD,
            DisplaySize {
                x: DISPLAY_WIDTH,
                y: DISPLAY_HEIGHT,
            },
        );

        // i2c1 construction (shared by the temperature sensor and RTC)
        let i2c1_bus = i2c1::init(res.peripherals.i2c1, res.i2c1, I2C1_FREQUENCY, &mut rcc);
        let i2c1_bus_static = Self::install_i2c1_singleton(i2c1_bus);
        let temperature = Temperature::new(
            CriticalSectionDevice::new(i2c1_bus_static),
            res.temperature_alert,
        );
        let rtc = Rtc::new(CriticalSectionDevice::new(i2c1_bus_static), res.rtc_mfp);

        // can construction
        let (can_instance, can_mode) = can::init(res.peripherals.can, res.can, &mut rcc);
        let can = CanBus::new(can_instance, can_mode);

        // vcom construction (USART3 to the FTDI USB-UART)
        let vcom_serial =
            usart3::init(res.peripherals.usart3, res.usart3, VCOM_BAUD_RATE, &mut rcc);
        let vcom = Vcom::new(vcom_serial, res.vcom_flow_control);

        // uart construction (USART4 to the UART connector)
        let uart = usart4::init(res.peripherals.usart4, res.usart4, UART_BAUD_RATE, &mut rcc);

        // usb construction (static, so classes and devices built from it can be 'static too)
        let usb_bus = usb::init(res.peripherals.usb, res.usb);
        let usb: &'static ApusUsbBus =
            singleton!(: ApusUsbBus = usb_bus).expect("failed to install usb singleton");

        // swd construction (already the debug port since reset)
        let swd = swd::init(res.swd);

        // led construction
        let leds = Leds::new(res.leds, res.peripherals.tim3, RGB_PWM_FREQUENCY, &mut rcc);

        // button construction
        let buttons = Buttons::new(res.buttons);

        // adc construction (shared by the potentiometer and the analog header pins)
        let adc = Adc::new(res.peripherals.adc, &mut rcc);

        // poti construction
        let potentiometer = Potentiometer::new(res.potentiometer);

        // buzzer construction
        let buzzer = Buzzer::new(res.buzzer);

        // user delay
        let delay = BusyDelay::new(sys_clock_hz);

        Self {
            leds,
            buttons,
            display,
            adc,
            potentiometer,
            buzzer,
            delay,
            touch,
            temperature,
            rtc,
            can,
            vcom,
            uart,
            usb,
            swd,

            pins: BoardPins {
                header: res.header,
                // analog mode for lower power consumption
                unconnected: res.unconnected.into_analog(),
            },
            peripherals: BoardPeripherals {
                configured: ConfiguredPeripherals { rcc },
                free: res.peripherals.free,
            },
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

pub struct BoardPins {
    pub header: ArduinoHeader,
    pub unconnected: UnconnectedPins<Analog>,
}

pub struct BoardPeripherals {
    pub configured: ConfiguredPeripherals,
    pub free: FreePeripherals,
}

pub struct ConfiguredPeripherals {
    pub rcc: rcc::Rcc,
}
