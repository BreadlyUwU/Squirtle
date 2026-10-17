mod clientmode;
mod standalone;
use squirtle_core::SYSTEM;
use crate::RunningMode;

pub fn start(mode: RunningMode) {
    match mode {
        RunningMode::Client => todo!(),
        RunningMode::Standalone => prepare_and_start_standalone(),
    }
}

fn prepare_and_start_clientmode() { todo!() }

/// Start Pidgey by initializing Squirtle's thermal subsystem, if it hasn't already been
fn prepare_and_start_standalone() {
    let current_uid = unsafe { libc::getuid() };
    if current_uid != 0 {
        eprintln!("Error: Pidgey in standalone mode must be launched with ROOT permissions.");
        std::process::exit(1);
    }
    let current_system = SYSTEM.clone();
    let current_thermal_system = current_system.read().unwrap().thermal.clone();
    match current_thermal_system {
        Some(thermal) => standalone::start(thermal),
        None => {
            log::info!("ThermalSystem hasn't been initialized yet, doing it now.");
            current_system.write().unwrap().init_thermalsystem();
            prepare_and_start_standalone();
        },
    }
}
