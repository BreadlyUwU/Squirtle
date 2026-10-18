use std::sync::{LazyLock, Arc, RwLock};

// mod cpu; // Not yet useful + Too old scrappy code
// mod gpu; // Not yet useful + Too old scrappy code
// mod lspci; // Dependency of cpu & gpu modules, so not usefull either
// mod msr; // Does not even exists what the hell are you doing here
pub mod thermal; // Thermal management backend
pub mod sysfs;
pub mod utils;

/// SYSTEM is the heart of squirtle-core. Once called, every components has to be initialized first.
/// - `init_cpuinfo()` method enable the CPU subsystem and store the current CPU informations (and will not do much else for now)
/// - `init_thermalsystem()` method enable Squirtle's thermal management system (the main protagonist of this library).
/// - *(More will "eventually" follow over time, once the most important parts are stabilized)*
///
/// init_thermalsystem() will create a new Arc & RwLock. You'll be able to store a SYSTEM-independant reference to it
/// so you can perform every operations you want in a thread-safe way, without locking the other subsystems.
///
/// The point of it is being accessible during Squirtle's daemon life-time and be interracted with via IPC by other
/// softwares (potentially by multiples ones and at the same time), starting with `pidgey` (Squirtle's fan-control daemon).
pub static SYSTEM: LazyLock<Arc<RwLock<GlobalSystem>>> = LazyLock::new(|| Arc::new(RwLock::new(GlobalSystem::new())) );

/// **GlobalSystem is the entry point of every Squirtle's subsystems. Once called first via `new()` method, you'll be able
/// to activate manually the components you need.**
///
/// This is to avoid loading a ton of informations in the same time if they're not immediately needed.
///
/// - [`init_cpuinfo()`] To enable Squirtle's CPU subsystem (Completely deactivated until it can be useful)
/// - [`init_thermal()`] To enable Squirtle's thermal subsystem
pub struct GlobalSystem {
    // pub cpu: Option<crate::cpu::CPUInfo>,
    // pub gpu: Option<sniffa::gpu::GPUInfo>,
    pub thermal: Option<Arc<RwLock<thermal::ThermalSystem>>>,
} impl GlobalSystem {
    pub fn new() -> Self {
        Self {
            // cpu: None,
            // gpu: None,
            thermal: None,
        }
    }

    // Return a CPUInfo struct full of little informations about the current CPU
    // pub fn init_cpuinfo(&mut self) { self.cpu = Some(crate::cpu::CPUInfo::get()) }

    /// Return a new Arc<RwLock<ThermalSystem>>>, from which you'll be able to start some operations on, e.g., the computer's fans.
    pub fn init_thermalsystem(&mut self) { self.thermal = Some(Arc::new(RwLock::new(thermal::ThermalSystem::new()))) }
}

pub fn version() -> String { String::from(env!("CARGO_PKG_VERSION")) }
