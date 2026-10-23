#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use panic_probe as _;

use apus::board::{defaults::RGB_PWM_FREQUENCY, resources::Resources};
use apus::bsp::{delay::BusyDelay, leds::Leds};
use apus::prelude::*;

use apus::hal::{
    adc::{Adc, VRef},
    dac::{self, DacOut, DacPin},
    pac,
};

// jumper A2 (DAC_OUT1) to A0 (ADC_IN1)
const DAC_MAX_CODE: u32 = (1 << 12) - 1;
// the output buffer does not reach the rails
const DAC_HEADROOM_MV: u16 = 200;
const DAC_SETTLING_US: u32 = 10;
const STEP_MV: u16 = 500;
const TOLERANCE_MV: u16 = 20;
const STEP_PERIOD_MS: u32 = 500;

/// DAC code that outputs `millivolts` with the reference at `vdda_mv`
fn dac_code(millivolts: u16, vdda_mv: u16) -> u16 {
    (u32::from(millivolts) * DAC_MAX_CODE / u32::from(vdda_mv)) as u16
}

#[entry]
fn main() -> ! {
    defmt::info!("APUS-RS example: header_dac_adc");

    let p = pac::Peripherals::take().unwrap();
    let mut res = Resources::new(p);

    let sys_clock_hz = res.rcc.clocks.sysclk().0;
    let mut delay = BusyDelay::new(sys_clock_hz);
    let mut leds = Leds::new(
        res.leds,
        res.peripherals.tim3,
        RGB_PWM_FREQUENCY,
        &mut res.rcc,
    );

    let header = res.header;
    let (dac_pin, mut adc_pin) =
        cortex_m::interrupt::free(|cs| (header.a2.into_analog(cs), header.a0.into_analog(cs)));

    let mut dac = dac::dac(res.peripherals.free.dac, dac_pin, &mut res.rcc);
    dac.enable();
    let mut adc = Adc::new(res.peripherals.adc, &mut res.rcc);

    loop {
        let vdda_mv = VRef::read_vdda(&mut adc);
        let highest_mv = vdda_mv - DAC_HEADROOM_MV;

        for target_mv in (DAC_HEADROOM_MV..=highest_mv).step_by(STEP_MV.into()) {
            dac.set_value(dac_code(target_mv, vdda_mv));
            delay.delay_us(DAC_SETTLING_US);
            let measured_mv = adc.read_abs_mv(&mut adc_pin);

            leds.main_leds_mut().into_iter().for_each(|led| led.off());

            // green within the tolerance, yellow outside (jumper missing)
            if measured_mv.abs_diff(target_mv) <= TOLERANCE_MV {
                defmt::info!("set {} mV, measured {} mV", target_mv, measured_mv);
                leds.led5.on();
            } else {
                defmt::error!("set {} mV, measured {} mV", target_mv, measured_mv);
                leds.led4.on();
            }

            delay.delay_ms(STEP_PERIOD_MS);
        }
    }
}
