use super::{PlatformProfile, Hwmon, HwmonEntry, Cooling, Temperature};
use std::{collections::HashMap, io::{Error, ErrorKind}};

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
    pub platform_profile: Option<Vec<PlatformProfile>>,
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

    fn get_available_platform_profiles() -> Option<Vec<PlatformProfile>> {
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
                Some(platforms_vec)
            },
            Err(e) => {
                log::warn!("Platform-Profile doesn't seem to be available on this platform. ({})", e);
                None
            },
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
