// Windwaker but in fully standalone mode (not using SquirtleD, only using Squirtle-Core as backend)

use std::{sync::{Arc, RwLock, atomic::{AtomicBool, Ordering}}, thread::sleep};
use squirtle_core::thermal::ThermalSystem;
use crate::CONFIGURATION;

enum TargetedCoolingMode { Active, Passive }
enum FanSpeedInterpolation { Step, Linear }

// MUST_EXIT defines whether Pidgey must completely shutdown itself (e.g.: when shuting down SquirtleD).
static MUST_EXIT: AtomicBool = AtomicBool::new(false);
// ENABLED define whether Pidgey will start and stop operations. When turned to `false`, Pidgey's internal threads
// will gracefully exits, but Pidgey will stay in standby mode.
// Can be eventually used in the future as a way to ask Pidgey to reload it's own config when, e.g., it is being modified
// by a GUI client.
static ENABLED: AtomicBool = AtomicBool::new(false);

/// Start Pidgey's fan-control daemon
pub fn start(arc: Arc<RwLock<ThermalSystem>>) {
    let system = arc.clone();
    log::info!("Starting Pidgey");

    // Let's fucking gooooo
    let orchestrator = std::thread::spawn(move || {
        log::info!("Let's a go!");

        // First storing the current platform_profile before trying anything
        let previous_platform_profile: String = {
            let platform_profile_guard = system.read().unwrap();
            platform_profile_guard.get_platform_profile("alienware-wmi")
        };
        log::debug!("previous_platform_profile set to {}", &previous_platform_profile);

        let platform_profile_restoration_needed: bool = if let Some(config_profile) = &CONFIGURATION.platform_profile {
            if &previous_platform_profile != config_profile {
                system.write().unwrap().set_platform_profile("alienware-wmi", config_profile.to_owned());
                log::info!("Platform Profile set to {}, previous state ({}) will be restored when exiting", &config_profile, &previous_platform_profile);
                true
            }
            else { false }
        } else { false };

        // Initializing the temperature sensors from the config against the discovered Hwmon interfaces
        if let Err(e) = initialize_temp_sensors(system.clone()) {
            log::error!("Unable to initialize the temperature sensors: {e}");
        }

        // Temperature surveillance thread
        let temp_system = system.clone();
        let temp_thread = std::thread::spawn(move || {
            log::info!("Starting the temperature surveillance");

            std::thread::spawn(move || {
                temperature_daemon(temp_system);
            }).join().expect("The temperature surveillance thread just crashed :c");

            log::info!("Temperature surveillance thread gracefully exited");
        });

        // Fan controller thread
        let fan_system = system.clone();
        let fan_thread = std::thread::spawn(move || {
            let fan_mode = match &CONFIGURATION.fan_update_interval {
                &Some(value) => if value > 0 { FanSpeedInterpolation::Linear} else { FanSpeedInterpolation::Step },
                &None            => FanSpeedInterpolation::Step,
            };
            log::info!("Starting the fan controller (enabled mode: {})", match &fan_mode {
                FanSpeedInterpolation::Step => "Step",
                FanSpeedInterpolation::Linear => "Linear",
            });

            std::thread::spawn(move || {
                fan_daemon(fan_system, fan_mode)
            }).join().expect("The fan controller thread just crashed :c");

            log::info!("Fan controller thread gracefully exited");
        });

        /********************************************************************/
        /* D */ sleep(std::time::Duration::from_secs(10));
        /* E */ println!("Finish!");
        /* B */ MUST_EXIT.store(true, Ordering::Relaxed);
        /* U */ fan_thread.join().unwrap();//_or(log::error!("Couldn't join the fan thread."));
        /* G */ temp_thread.join().unwrap();//_or(log::error!("Coudnl't join the temp thread."));
        /********************************************************************/

        // Restoring previous platform_profile if needed
        if platform_profile_restoration_needed == true {
            system.write().unwrap().set_platform_profile("alienware-wmi", previous_platform_profile);
            log::info!("Platform Profile restored.")
        }

        log::info!("Bye bye!");
    });

    // Let's fucking finiiiiish
    orchestrator.join().unwrap()
}

fn fan_daemon(system: Arc<RwLock<ThermalSystem>>,
              fan_interpolation_mode: FanSpeedInterpolation) {
    log::info!("Fan Daemon started.");

    let thermal_steps = &CONFIGURATION.thermal_targets;
    let mut current_step = 0;
    let mut cooling_mode = TargetedCoolingMode::Passive;
    let interval = match &fan_interpolation_mode {
        FanSpeedInterpolation::Step => CONFIGURATION.temp_check_interval,
        FanSpeedInterpolation::Linear => CONFIGURATION.fan_update_interval.unwrap(),
    };

    while !MUST_EXIT.load(Ordering::Relaxed) {
        sleep(std::time::Duration::from_secs(interval.into()));

        // TODO: DON'T FORGET TO RESET THE FANS AT THE THREAD'S EOL
    }
}

fn temperature_daemon(system: Arc<RwLock<ThermalSystem>>) {
    log::info!("Temperature refresh thread started.");

    let interval = CONFIGURATION.temp_check_interval.clone();

    while !MUST_EXIT.load(Ordering::Relaxed) {
        // system.write().unwrap().temperatures_refresh();
        // println!("Highlander: {}", system.read().unwrap().temperatures_highlander().unwrap());
        sleep(std::time::Duration::from_secs(interval.into()));
    }
}

// This is a pain in the ass for future myself
// fn fan_speed_interpolation() {}

/// Parse the `temperature_sources` map from the config, match every interface name against the Hwmon interfaces
/// already discovered inside the [`ThermalSystem`], then initialize `ThermalSystem::temperatures` with one
/// [`Temperature`] per sensor found.
fn initialize_temp_sensors(system: Arc<RwLock<ThermalSystem>>) -> std::io::Result<()> {
    Ok(()) // Bweeeh
}
