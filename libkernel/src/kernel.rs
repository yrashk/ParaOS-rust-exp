use crate::serial;
use spin::{Barrier, Once};

pub struct Kernel<'a> {
    bootstrap_processor_id: u32,
    start_barrier: &'a Once<Barrier>,
    quiet: bool,
    #[cfg(test)]
    bootstrap_initialized: bool,
}

impl<'a> Kernel<'a> {
    pub fn new(bootstrap_processor_id: u32, start_barrier: &'a Once<Barrier>) -> Self {
        Self {
            bootstrap_processor_id,
            start_barrier,
            quiet: false,
            #[cfg(test)]
            bootstrap_initialized: false,
        }
    }

    pub fn set_quiet(&mut self, quiet: bool) {
        self.quiet = quiet;
    }

    pub fn is_bootstrap_core(&self) -> bool {
        let cpuid = x86::cpuid::CpuId::new();
        let cpu_features = cpuid.get_feature_info().expect("CPU features");
        let local_apic = cpu_features.initial_local_apic_id() as u32;
        local_apic == self.bootstrap_processor_id
    }

    pub fn run(&mut self) {
        let cpuid = x86::cpuid::CpuId::new();
        let num_cores: u16 = crate::platform::num_cores();
        let start_rendezvous = self
            .start_barrier
            .call_once(|| Barrier::new(num_cores as usize));
        let cpu_features = cpuid.get_feature_info().expect("CPU features");
        let local_apic = cpu_features.initial_local_apic_id() as u32;
        let mut port = serial::Serial::new(&serial::COM1);
        if local_apic == self.bootstrap_processor_id {
            port.init();
            // Bootstrap CPU initialization
            #[cfg(test)]
            {
                self.bootstrap_initialized = true;
            }
        }
        start_rendezvous.wait();

        if local_apic == self.bootstrap_processor_id {
            if !self.quiet {
                core::fmt::write(&mut port, format_args!("ParaOS [{} cores]\n", num_cores))
                    .expect("serial output");
            }
        }
    }
}

mod tests {
    #[test_case]
    fn bootstrap_init_only() {
        use super::*;
        #[allow(non_upper_case_globals)]
        static barrier: Once<Barrier> = Once::new();
        let mut kernel = Kernel::new(0, &barrier);
        kernel.set_quiet(true);
        kernel.run();
        if kernel.is_bootstrap_core() {
            assert!(kernel.bootstrap_initialized)
        } else {
            assert!(!kernel.bootstrap_initialized);
        }
    }
}
