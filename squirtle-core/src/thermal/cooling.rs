use std::io::{Error, ErrorKind};
use std::path::PathBuf;

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
    pub fn get_fan_input(&mut self) -> std::io::Result<u16> {
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
        let value: u8 = crate::sysfs::sniffa(&path)?
            .parse()
            .map_err(|e| Error::new(ErrorKind::InvalidData, format!("Unable to parse the temperature value read at {:?}: {:?}", &path, e)))?;
        Ok(Temperature { source, path, value })
    }

    pub fn refresh(&mut self) -> std::io::Result<()> {
        match crate::sysfs::sniffa(&self.path) {
            Ok(v) => {
                self.value = v.parse().unwrap();
                Ok(())
            },
            Err(e) => Err(e),
        }
    }
}
