use embedded_hal::delay::DelayNs;

const NANOS_PER_SECOND: u64 = 1_000_000_000;

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
        let cycles = ((self.cpu_hz as u64) * (ns as u64)).div_ceil(NANOS_PER_SECOND);
        let actual_cycles = cycles / 2;
        cortex_m::asm::delay(actual_cycles as u32);
    }
}

// Note: we intentionally implement the newer `DelayNs` trait (embedded-hal v1)
// for BusyDelay above and pass BusyDelay directly into places that require
// an eh1-compatible delay implementation. Avoid referencing the old
// `embedded_hal::blocking` module here to prevent ambiguity between
// embedded-hal versions in the dependency graph.
