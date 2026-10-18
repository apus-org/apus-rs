use core::convert::Infallible;

use mipidsi::interface::{Interface, InterfaceKind};
use stm32f0xx_hal::gpio::{gpioe, Output, PushPull};
use stm32f0xx_hal::pac;

const DATA: u32 = 0xFF;
const D_C: u32 = 1 << 9;
const N_WR: u32 = 1 << 10;

/// `BSRR` bits that drive `pins` high
const fn set(pins: u32) -> u32 {
    pins
}

/// `BSRR` bits that drive `pins` low
const fn reset(pins: u32) -> u32 {
    pins << 16
}

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

/// 8080 bus of the display, two `BSRR` stores per byte.
///
/// DB0-DB7, D/C and WR all sit on GPIOE, so one store puts a byte on the data lines and
/// pulls WR low at the same time, and a second raises WR, which latches the byte.
pub struct DataBus {
    _pins: DataPins,
    _d_c: gpioe::PE9<Output<PushPull>>,
    _n_wr: gpioe::PE10<Output<PushPull>>,
}

impl DataBus {
    pub fn new(
        pins: DataPins,
        d_c: gpioe::PE9<Output<PushPull>>,
        n_wr: gpioe::PE10<Output<PushPull>>,
    ) -> Self {
        Self {
            _pins: pins,
            _d_c: d_c,
            _n_wr: n_wr,
        }
    }
}

#[inline(always)]
fn bsrr(bits: u32) {
    // SAFETY: BSRR is write-only and stateless. Only bits of the owned PE0-PE7, PE9 and
    // PE10 are ever set.
    unsafe {
        (*pac::GPIOE::ptr()).bsrr.write(|w| w.bits(bits));
    }
}

/// The store that drives `byte` onto DB0-DB7 and pulls WR low
#[inline(always)]
fn byte_with_wr_low(byte: u8) -> u32 {
    let byte = byte as u32;
    set(byte) | reset(!byte & DATA) | reset(N_WR)
}

#[inline(always)]
fn write(word: u32) {
    bsrr(word);
    bsrr(set(N_WR));
}

impl Interface for DataBus {
    type Word = u8;
    type Error = Infallible;

    const KIND: InterfaceKind = InterfaceKind::Parallel8Bit;

    fn send_command(&mut self, command: u8, args: &[u8]) -> Result<(), Infallible> {
        bsrr(reset(D_C));
        write(byte_with_wr_low(command));
        bsrr(set(D_C));

        for &arg in args {
            write(byte_with_wr_low(arg));
        }

        Ok(())
    }

    fn send_pixels<const N: usize>(
        &mut self,
        pixels: impl IntoIterator<Item = [u8; N]>,
    ) -> Result<(), Infallible> {
        for pixel in pixels {
            for byte in pixel {
                write(byte_with_wr_low(byte));
            }
        }

        Ok(())
    }

    fn send_repeated_pixel<const N: usize>(
        &mut self,
        pixel: [u8; N],
        count: u32,
    ) -> Result<(), Infallible> {
        let words = pixel.map(byte_with_wr_low);

        for _ in 0..count {
            for word in words {
                write(word);
            }
        }

        Ok(())
    }
}
