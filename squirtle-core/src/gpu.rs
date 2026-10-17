use super::lspci;

#[allow(unused)]
pub struct GPUInfo {
    vendor:          GPUVendor,
    gender:          GPUGender,
    card_name:       String,
    pci_addr:        String,
    card_id:         u8,
    render_id:       u8,
    memory:          Option<u16>, // I don't think the memory is revelant for integrated GPU
    clock_min_freq:  Option<u32>,
    clock_max_freq:  Option<u32>,
    memory_min_freq: Option<u32>,
    memory_max_freq: Option<u32>,
}

impl GPUInfo {}

#[allow(unused)]
enum GPUVendor {
    Unknown,
    NVIDIA,
    AMD,
    Intel
}

#[allow(unused)]
enum GPUGender { Integrated, Dedicated }

impl std::fmt::Display for GPUVendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GPUVendor::NVIDIA => write!(f, "Nvidia"),
            GPUVendor::AMD => write!(f, "AMD"),
            GPUVendor::Intel => write!(f, "Intel"),
            GPUVendor::Unknown => write!(f, "Unrecognized Vendor"),
        }
    }
}
impl std::fmt::Display for GPUGender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GPUGender::Dedicated => write!(f, "Dedicated"),
            GPUGender::Integrated => write!(f, "Integrated"),
        }
    }
}

// pub fn present_gpus() -> Vec<GPUInfo> {
//     let gpus: Vec<GPUInfo> = Vec::new();
//     let lspci_results = lspci::search_for("VGA");
//     for gpu in lspci_results {
//         let pci_addr = &gpu.split_whitespace().nth(0).unwrap().to_string();
//         gpus.push(pci);
//     }
//     return gpus
// }

// fn parse_vga_devices() { unimplemented!() }

pub fn demo() {
    println!("{:-^30}", "START DEMO GPU");
    println!("- lspci output for VGA devices:");
    for gpu in lspci::search_for("VGA") {
        println!("- {gpu}");
    }
    println!("- Attempt to parse CPU info for each lspci result:");
    for gpu in lspci::search_for("VGA") {
        let pci_addr = &gpu.split_whitespace().nth(0).unwrap().to_string();
        let card_name = &gpu.split("VGA compatible controller: ").nth(1).unwrap().to_string();
        println!("{pci_addr} - {card_name}");
    }
    println!("{:-^30}", "END DEMO GPU");
}
