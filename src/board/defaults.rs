use stm32f0xx_hal::time::{Bps, Hertz, KiloHertz};

/// Baud rate of [`Board::vcom`](super::Board::vcom)
pub const VCOM_BAUD_RATE: Bps = Bps(115_200);

/// Baud rate of [`Board::uart`](super::Board::uart)
pub const UART_BAUD_RATE: Bps = Bps(115_200);

/// PWM frequency of the RGB LED on TIM3
pub const RGB_PWM_FREQUENCY: Hertz = Hertz(1_000);

/// Clock of I2C1, shared by the temperature sensor and the RTC
pub const I2C1_FREQUENCY: KiloHertz = KiloHertz(100);

/// Clock of SPI1 to the touch controller (APB clock / 128)
pub const SPI1_FREQUENCY: KiloHertz = KiloHertz(375);
