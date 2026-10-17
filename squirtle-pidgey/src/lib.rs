// NOTE to myself: Don't even try to finish this part before finalizing SquirtleD (including every IPC calls)
// Also maybe at some point Windwaker should go to Squirtle-Core, so Pidgey and other clients will be mainly there
// for configuration purposes.
// I'm really not sure doing ~20 IPC call per second for quering temperatures & adjusting fan speed is a good thing.
#[allow(dead_code)]
mod config;
mod windwaker;
use squirtle_core::SYSTEM;
use config::CONFIGURATION;
use std::io::{self, Write};

pub enum RunningMode {
    Real,
    Simulation,
}

/// Start Pidgey by initializing Squirtle's thermal subsystem, if it hasn't already been
pub fn start(mode: RunningMode) {
    let current_system = SYSTEM.clone();
    let current_thermal_system = current_system.read().unwrap().thermal.clone();
    match current_thermal_system {
        Some(thermal) => windwaker::start(mode, thermal),
        None => {
            log::info!("ThermalSystem hasn't been initialized yet, doing it now.");
            current_system.write().unwrap().init_thermalsystem();
            start(mode);
        },
    }
}

pub fn test() {
    let current_system = SYSTEM.clone();
    let current_thermal_system = current_system.read().unwrap().thermal.clone();
    
    match current_thermal_system {
        Some(thermal) => {
            
            // First read of current platform-profile 
            print!("\r\x1b[2KTesting the SysFS writing: {:?}", thermal.read().unwrap().get_platform_profile("alienware-wmi"));
            io::stdout().flush().unwrap();
            std::thread::sleep(std::time::Duration::from_secs(2));

            // First switch of platform-profile to "performance"
            thermal.write().unwrap().set_platform_profile("alienware-wmi", "performance".to_string());
            print!("\r\x1b[2KTesting the SysFS writing: {:?}", thermal.read().unwrap().get_platform_profile("alienware-wmi"));
            io::stdout().flush().unwrap();
            std::thread::sleep(std::time::Duration::from_secs(5));

            // Switching back to "balanced"
            thermal.write().unwrap().set_platform_profile("alienware-wmi", "balanced".to_string());
            print!("\r\x1b[2KTesting the SysFS writing: {:?}", thermal.read().unwrap().get_platform_profile("alienware-wmi"));
            io::stdout().flush().unwrap();
            std::thread::sleep(std::time::Duration::from_secs(2));

            // Call it a day
            println!("\nDone.");
        },
        None => {
            current_system.write().unwrap().init_thermalsystem();
            test();
        },
    }
}

pub fn version() -> String { String::from(env!("CARGO_PKG_VERSION")) }