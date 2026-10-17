use std::{collections::HashMap, path::PathBuf, process::exit};
use serde::Deserialize;

const CONFIG_FILE_LOCATION: &str = "/etc/squirtle.d";
const CONFIG_FILE_NAME: &str = "pidgey.yml";

pub static CONFIGURATION: std::sync::LazyLock<Configuration> = std::sync::LazyLock::new(|| Configuration::init());

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub temp_check_interval: u8,
    pub global_hysteresis: u8,
    pub temperature_sources: Option<HashMap<String, Vec<String>>>,
    pub fan_update_interval: Option<u8>,
    pub platform_profile: Option<String>,
    pub thermal_targets: ThermalTargetType
}

#[derive(Debug, Deserialize)]
pub struct ThermalTarget {
    pub temp: u8,
    pub speed: ThermalTargetSpeedType,
    pub hysteresis: Option<u8>
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ThermalTargetType {
    General(Vec<ThermalTarget>),
    MultiFan(HashMap<ThermalTargetFanName, Vec<ThermalTarget>>),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ThermalTargetSpeedType {
    PercentageBased(u8),
    RPMBased(u16)
}

#[derive(Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(untagged)]
pub enum ThermalTargetFanName {
    Id(u8),
    Label(String),
}

// #[derive(Debug, Deserialize)]
// pub struct ThermalTargetMulti {
//     pub fan: ThermalTargetFanName,
//     pub thermal_targets: Vec<ThermalTarget>
// }

impl Configuration {
    pub fn init() -> Self {
        match Self::find_config_file() {
            Ok(file) => Self::read_config_file(file),
            Err(_) => {
                log::error!("Unable to find any config file.");
                // log::info!("Creating a new configuration file at: {}/{}", &CONFIG_FILE_LOCATION, &CONFIG_FILE_NAME);
                // create_new_configuration_file();
                // Self::init();
                exit(1);
            }
        }
    }

    fn read_config_file(file: PathBuf) -> Self {
        match std::fs::read_to_string(&file) {
            Ok(config) => {
                match yaml_serde::from_str::<Configuration>(config.as_str()) {
                    Ok(mut parsed_config) => {
                        match &mut parsed_config.thermal_targets {
                            ThermalTargetType::General(thermal_targets) => normalize_target_speed(thermal_targets),
                            ThermalTargetType::MultiFan(thermal_target_multis) => {
                                for targets in thermal_target_multis.values_mut() {
                                    normalize_target_speed(targets);
                                }
                            },
                        }
                        log::debug!("Configuration ({}) succesfully parsed.", &file.into_string().unwrap());
                        return parsed_config 
                    },
                    Err(e) => {
                        eprintln!("\nThere's an error inside your current config file. Please fix it.\n{}: {:?}\n", file.display(), e);
                        exit(1);
                    }
                }
            }
            Err(e) => { 
                eprintln!("Cannot open the config file {}: {:?}", file.display(), e);
                exit(1);
            }
        }
    }

    fn find_config_file() -> Result<PathBuf, bool> {
        let config_file_etc = std::path::Path::new(&CONFIG_FILE_LOCATION).join(&CONFIG_FILE_NAME);
        match config_file_etc.exists() {
            true  => { return Ok(config_file_etc.to_path_buf()) },
            false => {
                let config_file_local = std::path::Path::new(CONFIG_FILE_NAME);
                match config_file_local.exists() {
                    true  => return Ok(config_file_local.to_path_buf()),
                    false => return Err(false)
                }
            }
        }
    }

    // fn default_config() -> Self {
    //     let temp_check_interval: u8 = 2;
    //     let fan_update_interval: Option<u8> = None;
    //     let platform_profile: Option<String> = None;
    //     let global_hysteresis: u8 = 5;
    //     let mut thermal_targets: Vec<ThermalTarget> = Vec::new();
    //     thermal_targets.push(ThermalTarget {temp: 60, boost: 40, hysteresis: Some(10)});
    //     thermal_targets.push(ThermalTarget {temp: 65, boost: 55, hysteresis: None});
    //     thermal_targets.push(ThermalTarget {temp: 70, boost: 70, hysteresis: None});
    //     thermal_targets.push(ThermalTarget {temp: 75, boost: 85, hysteresis: None});
    //     thermal_targets.push(ThermalTarget {temp: 80, boost: 100, hysteresis: None});

    //     Configuration { temp_check_interval, global_hysteresis, fan_update_interval, platform_profile, thermal_targets: ThermalTargetType::General(thermal_targets) }
    // }
}

fn normalize_target_speed(targets: &mut Vec<ThermalTarget>) {
    for target in targets {
        if let ThermalTargetSpeedType::PercentageBased(speed) = target.speed {
            if speed > 100 { target.speed = ThermalTargetSpeedType::PercentageBased(100) }
        }
    }
}

fn create_new_configuration_file() {}