use std::{collections::HashMap, io::{Error, ErrorKind}, path::PathBuf};

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
