use std::{
    collections::HashMap, 
    path::PathBuf,
    io::{
        Error, 
        ErrorKind, 
    }
};

use crate::sysfs;

/// The Thermal System is an global interface accessible as `Arc<RwLock<ThermalSystem>>` via the [`squirtle_core::SYSTEM`] LazyLock to store 
/// information about and manipulate:
/// 
/// - The computer's currently used platform profile
/// - A list of every available Hwmon interfaces at runtime
/// - The computer's current temperatures
/// - The computer's cooling system
/// 
/// Once ThermalSystem is ready, both `temperatures` and `cooling` will need to be manually initialized.
/// by the client, depending on user's configuration and what has been found inside `hwmon_interfaces`.
pub struct ThermalSystem {
    pub platform_profile: Vec<PlatformProfile>,
    pub hwmon_interfaces: HashMap<String, Hwmon>,
    /// Must be defined by the client, Squirtle-core cannot know in advance where it should look at.
    pub temperatures: Option<Vec<Temperature>>,
    /// Must be defined by the client, Squirtle-core cannot know in advance where it should look at.
    pub cooling: Option<Cooling>,
} impl ThermalSystem {
    /// ThermalSystem::new() is used when initializing squirtle_core::SYSTEM's LazyLock.
    pub fn new() -> Self {
        ThermalSystem {
            platform_profile: Self::get_available_platform_profiles(),
            hwmon_interfaces: Self::get_available_hwmon_interfaces(),
            temperatures: None,
            cooling: None
        }
    }

    /// Returns a simple string of the current platform_profile used by the computer.
    /// 
    /// The function needs to know which platform provider you want to query.  
    /// For instance, on a Dell G-Series laptop, you'll probably want to deal with `alienware-wmi` instead of `dell-pc`.
    pub fn get_platform_profile(&self, platform: &str) -> String {
        let selected_platform: &PlatformProfile = match self.platform_profile.iter().find(|p| p.source.as_str() == platform) {
            Some(platform) => platform,
            None => panic!("Tried to gather info. about the platform-profile from a provider that hasn't been found on this device."),
        };
        selected_platform.profile.clone()
    }

    /// Set the wanted profile to the wanted platform_profile provider.
    /// 
    /// The wanted profile will be tested against the user PC's availables choices before setting anything.
    /// Will log an error if the wanted profile doesn't exists in the first place, same thing for the wanted platform.
    pub fn set_platform_profile(&mut self, platform: &str, wanted: String) {
        let selected_platform: &mut PlatformProfile = match self.platform_profile.iter_mut().find(|p| p.source.as_str() == platform) {
            Some(platform) => platform,
            None => {
                log::error!("Tried to set a platform-profile to a platform that hasn't been found on this device.");
                return;
            }
        };
        selected_platform.set_profile(wanted);
    }

    /// Refresh the latest known temperatures inside the struct
    pub fn temperatures_refresh(&mut self) {
        if let Some(sensors) = self.temperatures.as_mut() {
            for sensor in sensors.iter_mut() {
                let _ = sensor.refresh();
            } 
        } else { log::error!("Tried to refresh the current temperature(s) depsite it/them not being proprely initialized."); }
    }

    /// Returns the highest temperature reported by every initialized sensor.
    pub fn temperatures_highlander(&self) -> std::io::Result<u8> {
        if let Some(sensors) = &self.temperatures {
            sensors.iter()
                .map(|sensor| sensor.value)
                .max()
                .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Something went wrong while iterating over the current watched temperatures to find the highest value."))
        } else {
            Err(Error::new(ErrorKind::NotFound, "Tried to compare the temperatures from every watched sensors… but there's no initialized sensors."))
        }
    }

    /// Refresh the latest known fan(s) current speed
    pub fn current_fan_input_refresh(&mut self) { 
        if let Some(devices) = &mut self.cooling {
            for device in devices.devices.iter_mut() {
                if let Err(e) = device.get_fan_input() {
                    log::warn!("Unable to refresh fan input for {} (id: {}): {}", device.source, device.id, e);
                }
            }
        } else { log::error!("Tried to refresh the current fan(s) input depsite it/them not being properly initialized."); }
    }
    
    fn get_available_platform_profiles() -> Vec<PlatformProfile> {
        match std::fs::read_dir("/sys/class/platform-profile") {
            Ok(platforms) => {
                let mut platforms_vec: Vec<PlatformProfile> = Vec::new();
                for platform in platforms.flatten() {
                    let name = crate::sysfs::sniffa(platform.path().join("name")).unwrap();
                    let profile = crate::sysfs::sniffa(platform.path().join("profile")).unwrap();
                    let choices: Vec<String> = crate::sysfs::sniffa(platform.path().join("choices")).unwrap()
                        .split_whitespace().map(ToString::to_string).collect();
                    platforms_vec.push(PlatformProfile { location: platform.path(), source: name, profile, choices });
                }
                platforms_vec
            },
            Err(e) => panic!("{:?}", e),
        }
    }

    fn get_available_hwmon_interfaces() -> HashMap<String, Hwmon> {
        match std::fs::read_dir("/sys/class/hwmon") {
            Ok(interfaces) => {
                let mut global_hwmon_hashmap: HashMap<String, Hwmon> = HashMap::new(); 
                for interface in interfaces.flatten() {
                    let interface_name: String = crate::sysfs::sniffa(interface.path().join("name")).unwrap();
                    let mut interface_content: HashMap<String, HwmonEntry> = HashMap::new();
                    for file in interface.path().read_dir().unwrap().flatten() {
                        let readonly: bool = { 
                            let file_metadata = file.path().metadata().unwrap();
                            file_metadata.permissions().readonly() 
                        };
                        interface_content.insert(
                            file.file_name().into_string().unwrap(), 
                            HwmonEntry { path: file.path(), readonly }
                        );
                    }
                    global_hwmon_hashmap.insert(
                        interface_name, 
                        Hwmon { path: interface.path(), values: interface_content }
                    );
                }
                return global_hwmon_hashmap // Pfiou!
            } Err(e) => panic!("{e}") 
        }
    }
} 

pub struct PlatformProfile {
    pub source: String,
    location: std::path::PathBuf,
    pub profile: String,
    pub choices: Vec<String>,
} impl PlatformProfile {
    fn set_profile(&mut self, wanted: String) {
        if self.do_wanted_profile_exists_in_the_first_place(&wanted) {
            if let Ok(()) = crate::sysfs::boop(self.location.clone().join("profile"), wanted.clone()) {
                log::info!("Platform-profile ({}) set to \"{}\"", self.source, &wanted);
                self.profile = wanted;
            } else { log::error!("Unable to set platform-profile: {}, {}", self.source, wanted) }
        } else {
            log::error!("The wanted profile ({}) doesn't exists for this platform ({}). Here's the available choices: {:?}", wanted, self.source, self.choices);
        }
    }

    fn do_wanted_profile_exists_in_the_first_place(&self, wanted: &String) -> bool { self.choices.contains(wanted) }
}

/// `Hwmon` represents a single Hwmon interface, it's path, and a list of every single values provided by it and their associated 
/// path, and write premissions status, both inside a HashMap.
/// 
/// It will be used both for knowing where the program should write any SysFS value for, e.g. changing the fan speed,
/// as a mean to know if an interface exists in the first place or helping with finding actionnable SysFS values.
/// 
/// The point of it is fetching everything's available once and keep them in memory for a faster access.
/// 
/// Every `Hwmon` structs are distributed inside a HashMap (with the interface label as the key) inside ThermalSystem. 
pub struct Hwmon {
    pub path: PathBuf,
    pub values: HashMap<String, HwmonEntry>,
} impl Hwmon {
    /// Return the SysFS path associated with a specific value (e.g. "temp1_input") provided by this Hwmon interface.
    pub fn get_path(&self, req: &str) -> std::io::Result<PathBuf> {
        match self.values.get(req) {
            Some(entry) => Ok(entry.path.clone()),
            None => Err(Error::new(ErrorKind::NotFound, format!("The requested SysFS value ({}) doesn't exists on this Hwmon interface.", req))),
        }
    }
}

pub struct HwmonEntry {
    pub path: PathBuf,
    pub readonly: bool
}

pub struct Cooling {
    pub mode: CoolingMode,
    pub devices: Vec<CoolingDevice>,
} impl Cooling {
    fn get_available_devices(hwmon: std::path::PathBuf) -> Vec<CoolingDevice> {
        // Sur quoi on se base pour savoir combien de ventrads a le PC ?
        // Peut-être besoin de support de wildcards, ou alors se baser sur le nombre d'entrées contenant "pwm*" (pwmX_auto_channels_temp)
        let mut entries_vec: Vec<CoolingDevice> = Vec::new();
        match std::fs::read_dir(&hwmon) {
            Ok(entries) => {
                let mut entry_id: u8 = 0;
                for _ in entries.flatten().filter(|entry| {entry.file_name().into_string().unwrap().contains("pwm")}) {
                    entry_id += 1;
                    // let hwmon = &hwmon;
                    let location = hwmon.to_path_buf();
                    let id = entry_id.clone();
                    let source = crate::sysfs::sniffa(&hwmon.join("name")).unwrap();
                    let label: Option<String> = match crate::sysfs::sniffa(&hwmon.join(format!("fan{}_label", &id))) {
                        Ok(value) => Some(value),
                        Err(_) => None,
                    };
                    // let min_speed: u16 = crate::sysfs::sniffa(&hwmon.join(format!("fan{}_min", &id))).parse().unwrap_or(0);
                    let fan_min: Option<u16> = match crate::sysfs::sniffa(&hwmon.join(format!("fan{}_min", &id))) {
                        Ok(value) => Some(value.parse::<u16>().unwrap_or(0)),
                        Err(_) => None,
                    };
                    // let max_speed: u16 = crate::sysfs::sniffa(&hwmon.join(format!("fan{}_max", &id))).parse().unwrap_or(0);
                    let fan_max: Option<u16> = match crate::sysfs::sniffa(&hwmon.join(format!("fan{}_max", &id))) {
                        Ok(value) => Some(value.parse::<u16>().unwrap_or(0)),
                        Err(_) => None,
                    };
                    // let current_speed: u16 = crate::sysfs::sniffa(&hwmon.join(format!("fan{}_input", &id))).parse().unwrap_or(0);
                    let fan_input: Option<u16> = match crate::sysfs::sniffa(&hwmon.join(format!("fan{}_input", &id))) {
                        Ok(value) => Some(value.parse::<u16>().unwrap_or(0)),
                        Err(_) => None,
                    };
                    // let boost: u8 = crate::sysfs::sniffa(&hwmon.join(format!("fan{}_boost", &id))).parse().unwrap_or(0);
                    let fan_boost: Option<u8> = match crate::sysfs::sniffa(&hwmon.join(format!("fan{}_boost", &id))) {
                        Ok(value) => Some(value.parse::<u8>().unwrap_or(0)),
                        Err(_) => None,
                    };
                    let fan_target: Option<u16> = match crate::sysfs::sniffa(&hwmon.join(format!("fan{}_target", &id))) {
                        Ok(value) => Some(value.parse::<u16>().unwrap_or(0)),
                        Err(_) => None,
                    };
                    entries_vec.push(
                        CoolingDevice { location, source, id, label, fan_input, fan_boost, fan_min, fan_max, fan_target }
                    );
                }
            },
            Err(e) => panic!("{e}"),
        }
        return entries_vec
    }
}

pub struct CoolingDevice {
    /// The Hwmon interface this cooling device originates from (not to be confused with `location`)
    pub source: String,
    /// The current SysFS path to the Hwmon interface where this cooling device is accessible
    location: PathBuf,
    /// The ID of the cooling device.
    /// E.g.: Inside `dell_smm`'s Hwmon interface, it is `fan1_*`, or `fan2_*`? 
    pub id: u8,
    /// The name of this device as reported by it's Hwmon interface.
    /// E.g.: `alienware_wmi` will usually report "CPU Fan" for the first fan device. 
    pub label: Option<String>,
    pub fan_min: Option<u16>,
    pub fan_max: Option<u16>,
    pub fan_target: Option<u16>,
    /// Not an Option<u16> because I'm assuming `fanX_input` will always be around, unlike `fanX_boost` 
    /// (it would be really weird otherwise, Squirtle panicking right here should be the most sanest behavior for now if it's the case)
    pub fan_input: Option<u16>,
    /// "boost" as far as I understand it is pretty specific to the `alienware_wmi` kernel module, it will likely not exist on most devices.
    /// Hence the precautions about it.
    pub fan_boost: Option<u8>,
} impl CoolingDevice {
    /// Return the current device's speed input 
    fn get_fan_input(&mut self) -> std::io::Result<u16> {
        if self.fan_input != None {
            let fan_input_path = format!("fan{}_input", self.id);
            let input = crate::sysfs::sniffa(self.location.join(fan_input_path)).unwrap().parse::<u16>().unwrap();
            self.fan_input = Some(input);
            Ok(input) 
        } else { Err(Error::new( ErrorKind::Unsupported, format!("There's no 'input' entry to read on for this cooling device: {} (id: {}).", self.source, self.id))) }
    }

    /// Return the current device's target speed.
    fn get_fan_target(&mut self) -> std::io::Result<u16> {
        if self.fan_target != None {
            let fan_target_path = format!("fan{}_target", self.id);
            let target = crate::sysfs::sniffa(self.location.join(fan_target_path)).unwrap().parse::<u16>().unwrap();
            self.fan_target = Some(target);
            Ok(target)
        } else { Err(Error::new( ErrorKind::Unsupported, format!("There's no 'target' entry to read on for this cooling device: {} (id: {}).", self.source, self.id))) }
    }

    /// Write a new speed target value, assuming a "fan*_target" value exits for this device and is writable.
    fn set_fan_target(&mut self, val: u16) -> std::io::Result<u16> {
        if self.fan_target != None {
            let fan_target_path = format!("fan{}_target", self.id);
            match crate::sysfs::boop(self.location.join(fan_target_path), val) {
                Ok(()) => { self.fan_target = Some(val); Ok(val) },
                Err(e) => Err(e),
            }
        } else { Err(Error::new(ErrorKind::Unsupported, format!("There's no 'target' entry to write on for this cooling device: {} (id: {}).", self.source, self.id))) }
    }

    /// Return the current device's boost input (useful for, e.g., cooling devices controlled by `alienware_wmi`).
    fn get_fan_boost(&mut self) -> std::io::Result<u8> {
        if self.fan_boost != None {
            let fan_boost_path = format!("fan{}_boost", self.id);
            let boost = crate::sysfs::sniffa(self.location.join(fan_boost_path)).unwrap().parse::<u8>().unwrap();
            self.fan_boost = Some(boost);
            Ok(boost)
        } else { Err(Error::new( ErrorKind::Unsupported, format!("There's no 'boost' entry to read on for this cooling device: {} (id: {}).", self.source, self.id))) }
    }

    /// Write a new boost value, if a "fan*_boost" value exits for this device and is writable.
    fn set_fan_boost(&mut self, val: u8) -> std::io::Result<u8> {
        if self.fan_boost != None {
            let fan_boost_path = format!("fan{}_boost", self.id);
            match crate::sysfs::boop(self.location.join(fan_boost_path), val) {
                Ok(()) => { self.fan_boost = Some(val); Ok(val) },
                Err(e) => Err(e),
            }
        } else { Err(Error::new(ErrorKind::Unsupported, format!("There's no 'boost' entry to write on for this cooling device: {} (id: {}).", self.source, self.id))) }
    }
}

pub enum CoolingMode { Active, Passive }
#[derive(Debug)]
pub struct Temperature {
    pub source: String,
    path: PathBuf,
    pub value: u8,
} impl Temperature {
    /// Build a new [`Temperature`] sensor from a SysFS path, immediately reading and storing its current value.
    /// 
    /// The `source` is the name of the Hwmon interface this sensor originates from (e.g. `dell_smm`).
    pub fn new(source: String, path: PathBuf) -> std::io::Result<Temperature> {
        let value: u8 = sysfs::sniffa(&path)?
            .parse()
            .map_err(|e| Error::new(ErrorKind::InvalidData, format!("Unable to parse the temperature value read at {:?}: {:?}", &path, e)))?;
        Ok(Temperature { source, path, value })
    }

    pub fn refresh(&mut self) -> std::io::Result<()> {
        match sysfs::sniffa(&self.path) {
            Ok(v) => {
                self.value = v.parse().unwrap();
                Ok(())
            },
            Err(e) => Err(e),
        }
    }
}
