use embedded_hal::delay::DelayNs;

pub struct BusyDelay {
    cpu_hz: u32,
}

impl BusyDelay {
    pub fn new(cpu_hz: u32) -> Self {
        BusyDelay { cpu_hz }
    }
}

impl DelayNs for BusyDelay {
    fn delay_ns(&mut self, ns: u32) {
        // Approximate number of CPU cycles for the requested delay
        let cycles = ((self.cpu_hz as u64) * (ns as u64) + 999_999_999) / 1_000_000_000;
        let actual_cycles = cycles / 2;
        cortex_m::asm::delay(actual_cycles as u32);
    }
}