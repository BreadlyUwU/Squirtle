use std::{fs, io::ErrorKind, sync::LazyLock};
use super::sysfs;
pub static CPUINFO: LazyLock<String> = LazyLock::new(get_cpuinfo_once);

pub enum CPUVendor {
    Intel,
    AMD,
    Unknown,
}
impl std::fmt::Display for CPUVendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CPUVendor::AMD => write!(f, "AMD"),
            CPUVendor::Intel => write!(f, "Intel"),
            CPUVendor::Unknown => write!(f, "Unrecognized Vendor"),
        }
    }
}

pub struct CPUInfo {
    vendor:    CPUVendor,
    model:     String,
    cores:     u8,
    threads:   u8,
    base_freq: u32, // as Hz
    max_freq:  u32,
    flags:     Vec<String>,
}

impl CPUInfo {
    pub fn get() -> Self {
        CPUInfo {
            vendor:    Self::vendor(),
            model:     Self::model(),
            cores:     Self::total_physical_cores(),
            threads:   Self::total_logical_cores(),
            base_freq: Self::base_freq(),
            max_freq:  Self::max_freq(),
            flags:     Self::flags(),
        }
    }

    pub fn show(&self) {
        println!("{:-^30}", "== CPU Info ==");
        println!("- Vendor: {}", self.vendor);
        println!("- Model name: {}", self.model);
        println!("- Nbr. of physical cores: {}", self.cores);
        println!("- Nbr. of logical cores: {}", self.threads);
        println!("- Base frequency: {}", self.base_freq);
        println!("- Max freq. (assumed from core 0): {}", self.max_freq);
        println!("- Max freq. (for each cores):");
        for i in CPUInfo::max_freq_multi() { println!("---- Cpu {}: {}", i.0, i.1); }
        // println!("- Status of Intel HWP Dynamic Boost: {}",
        //     if CPUInfo::hwp_dyn_boost_status() { "Enabled" } else { "Disabled" }
        // );
        println!("- CPU flags: {:?}", self.flags);
        println!("{:-^30}\n", "== End CPU Info ==");
    }

    fn vendor() -> CPUVendor {
        let result = search_proc_cpuinfo("vendor_id");
        match result.as_str() {
            "GenuineIntel" => CPUVendor::Intel,
            "AuthenticAMD" => CPUVendor::AMD,
            _ => CPUVendor::Unknown,
        }
    }

    fn model() -> String { search_proc_cpuinfo("model name") }
    fn base_freq() -> u32 { sysfs::sniffa("/sys/devices/system/cpu/cpu0/cpufreq/base_frequency").unwrap().parse().unwrap() }
    fn max_freq() -> u32 { sysfs::sniffa("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq").unwrap().parse().unwrap() }
    fn max_freq_multi() -> Vec<(u8, u32)> {
        let mut values: Vec<(u8, u32)> = Vec::new();
        for cpu in 0..CPUInfo::total_logical_cores() {
            let value_got: u32 = sysfs::sniffa(format!("/sys/devices/system/cpu/cpu{cpu}/cpufreq/cpuinfo_max_freq")).unwrap().parse().unwrap();
            values.push((cpu, value_got));
        };
        return values;
    }
    fn total_physical_cores() -> u8 { return search_proc_cpuinfo("cpu cores").parse::<u8>().unwrap() }
    fn total_logical_cores() -> u8 { return search_proc_cpuinfo("siblings").parse::<u8>().unwrap() }
    fn flags() -> Vec<String> {
        let mut values: Vec<String> = Vec::new();
        for item in search_proc_cpuinfo("flags").split_whitespace() {
            values.push(item.to_string());
        }
        return values;
    }
    // Disabled because not vendor-agnostic
    // TODO: Maybe I could later wrap informations like theses inside the CPUVendor enum
    // But really not a priority for now
    // fn hwp_dyn_boost_status() -> bool {
    //     match sysfs::sniffa("/sys/devices/system/cpu/intel_pstate/hwp_dynamic_boost").unwrap().as_str() {
    //         "1" => return true,
    //         _   => return false,
    //     }
    // }

}

fn get_cpuinfo_once() -> String {
    match fs::read_to_string("/proc/cpuinfo") {
        Ok(cpuinfo) => cpuinfo,
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => {
                    eprintln!("Failed to read cpuinfo: /proc/cpuinfo doesn't exist.");
                    std::process::exit(21);
                },
                ErrorKind::PermissionDenied => {
                    eprintln!("Failed to read cpuinfo: /proc/cpuinfo read permission denied.");
                    std::process::exit(22);
                },
                _ => {
                    eprintln!("Unexpected error while trying to read /proc/cpuinfo:\n{e:?}");
                    std::process::exit(20);
                }
            }
        }
    }
}

fn search_proc_cpuinfo(search: &str) -> String {
    let cpuinfo_search = CPUINFO
        .lines()
        .find(|line| line.starts_with(search))
        .and_then(|line| line.split(':').nth(1).map(|value| value.trim().to_string()));
    match cpuinfo_search {
        Some(result) => return result,
        None => return "Err".to_string(),
    }
}
