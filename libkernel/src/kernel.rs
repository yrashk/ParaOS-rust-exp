use crate::serial;
use spin::{Barrier, Once};

pub struct Kernel<'a> {
    bootstrap_processor_id: u32,
    start_barrier: &'a Once<Barrier>,
}

impl<'a> Kernel<'a> {
    pub fn new(bootstrap_processor_id: u32, start_barrier: &'a Once<Barrier>) -> Self {
        Self {
            bootstrap_processor_id,
            start_barrier,
        }
    }
    pub fn run(&mut self) {
        let cpuid = x86::cpuid::CpuId::new();
        let num_cores: u16 = cpuid
            .get_extended_topology_info()
            .expect("Extended topology info")
            .filter(|i| i.level_type() == x86::cpuid::TopologyType::Core)
            .map(|i| i.processors())
            .sum();
        let start_rendezvous = self
            .start_barrier
            .call_once(|| Barrier::new(num_cores as usize));
        let cpu_features = cpuid.get_feature_info().expect("CPU features");
        let local_apic = cpu_features.initial_local_apic_id() as u32;
        let mut port = serial::Serial::new(&serial::COM1);
        if local_apic == self.bootstrap_processor_id {
            port.init();
            // Bootstrap CPU initialization
        }
        start_rendezvous.wait();
        if local_apic == self.bootstrap_processor_id {
            core::fmt::write(&mut port, format_args!("ParaOS [{} cores]\n", num_cores))
                .expect("serial output");
        }
    }
}
