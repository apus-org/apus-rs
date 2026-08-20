use core::convert::Infallible;

use mipidsi::interface::{InterfaceKind, OutputBus};
use stm32f0xx_hal::gpio::{gpioe, Output, PushPull};
use stm32f0xx_hal::pac;

/// DB0-DB7 of the display, wired to PE0-PE7
pub struct DataPins {
    pub db0: gpioe::PE0<Output<PushPull>>,
    pub db1: gpioe::PE1<Output<PushPull>>,
    pub db2: gpioe::PE2<Output<PushPull>>,
    pub db3: gpioe::PE3<Output<PushPull>>,
    pub db4: gpioe::PE4<Output<PushPull>>,
    pub db5: gpioe::PE5<Output<PushPull>>,
    pub db6: gpioe::PE6<Output<PushPull>>,
    pub db7: gpioe::PE7<Output<PushPull>>,
}

/// 8-bit parallel data bus, one `BSRR` store per byte.
///
/// The data lines are contiguous on PE0-PE7, so a byte fits a single store: the
/// low half-word raises the set bits, the high half-word lowers the rest.
pub struct DataBus {
    _pins: DataPins,
}

impl DataBus {
    pub fn new(pins: DataPins) -> Self {
        Self { _pins: pins }
    }
}

impl OutputBus for DataBus {
    type Word = u8;
    type Error = Infallible;

    const KIND: InterfaceKind = InterfaceKind::Parallel8Bit;

    #[inline(always)]
    fn set_value(&mut self, value: u8) -> Result<(), Infallible> {
        let value = value as u32;
        let bits = value | ((!value & 0xFF) << 16);

        // SAFETY: BSRR is write-only and stateless. Bits 8..16 and 24..32 stay
        // clear, so only the owned PE0-PE7 are driven.
        unsafe {
            (*pac::GPIOE::ptr()).bsrr.write(|w| w.bits(bits));
        }

        Ok(())
    }
}
