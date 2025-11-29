use tsc2046::Tsc2046;

pub const DEFAULT_THRESHOLD : f32 = 0.5;    // TODO: find optimal value

/// Concrete configured touch driver type used by the board.
pub type Touch = Tsc2046<
	embedded_hal_bus::spi::ExclusiveDevice<
		embedded_hal_compat::Forward<
			shared_bus::SpiProxy<'static, shared_bus::NullMutex<crate::peripherals::spi::spi1::Spi1>>
		>,
		embedded_hal_compat::Forward<
			stm32f0xx_hal::gpio::gpioe::PE12<stm32f0xx_hal::gpio::Output<stm32f0xx_hal::gpio::PushPull>>,
			embedded_hal_compat::markers::ForwardOutputPin
		>,
		crate::bsp::delay::BusyDelay,
	>,
>;

/// Initialize a `Tsc2046` from a prepared SPI device.
///
/// Responsibility: caller must prepare a SPI device (e.g. an
/// `ExclusiveDevice` wrapping a shared-bus proxy and a CS pin) and pass it
/// here. This keeps construction logic for SPI devices outside the BSP.
pub fn init<D>(device: D, touch_threshold : f32) -> Tsc2046<D>
where
	D: embedded_hal::spi::SpiDevice + embedded_hal::spi::ErrorType,
{
	Tsc2046::new(device, true, touch_threshold).unwrap()
}
