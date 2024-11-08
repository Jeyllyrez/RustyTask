use nvml_wrapper::Nvml;
use nvml_wrapper::error::NvmlError;
use sysinfo::{System, Disks, Networks};
use std:: {thread, time };
use std::time::Duration;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::fs::OpenOptions;
use chrono::Local;

pub fn gpu_usage() -> Result<Vec<u64>, NvmlError> { // go unwrap urself
    let mut result: Vec<u64> = Vec::new();
    let nvml = Nvml::init()?;
    let device_count = nvml.device_count()?;

    for i in 0..device_count {
        let device = nvml.device_by_index(i)?;
        let utilization = device.utilization_rates()?;
        // let percentage = gpu::gpu_usage(&nvml, i)?;
        // println!("GPU {} usage: {:?}%", i, percentage);
        result.push(utilization.gpu as u64)
    }    
    
    Ok(result)
}

pub fn cpu_usage() -> u64 {
    let mut sys = System::new_all();

    sys.refresh_cpu_all();
    thread::sleep(Duration::from_secs(1));
    sys.refresh_cpu_all();

    sys.global_cpu_usage() as u64
}

pub fn disk_usage() -> Vec<(String,u64)> {
    let disks = Disks::new_with_refreshed_list();
    let mut all_disk = Vec::new();
    for disk in disks.list() {
        let temp = (disk.total_space() - disk.available_space()) * 100 / disk.total_space();
        let name = match disk.name().to_str() {
            Some(n) => n,
            None => "none",
        };
        all_disk.push((name.to_string(), temp));
    }
    all_disk
}
pub fn disk_info() -> (u64, u64, u64) {
    let mut disks = Disks::new_with_refreshed_list();
    let (mut percent, mut used, mut space) = (0, 0, 0);
    for (_, usage) in disk_usage() {
        percent += usage;
    }
    percent /= disk_usage().len() as u64;
    for disk in disks.list_mut() {
        disk.refresh();
        used += disk.total_space() / 1073741824 - disk.available_space() / 1073741824;
        space += disk.total_space() / 1073741824;
    }
    (percent, used, space)
}

pub fn network_usage() -> Vec<(u64, u64)> { // (receive, transmit) in bytes, go change urself
    let mut all_network = Vec::new();

    let mut networks = Networks::new_with_refreshed_list();
    thread::sleep(time::Duration::from_millis(10));
    networks.refresh();
    for (_, network) in &networks {
        all_network.push((network.received() * 8, network.transmitted() * 8));
    }

    all_network
}

pub fn log_file() {
    let mut content = String::new();

    // Get the current time
    let current_time = Local::now();
    content.push_str(&format!("[{}]\n", current_time.format("%Y-%m-%d %H:%M:%S")));

    // CPU usage
    content.push_str(&format!("CPU usage: {}%\n", cpu_usage()));

    // GPU usage
    if let Ok(gpu_usages) = gpu_usage() {
        for (i, usage) in gpu_usages.iter().enumerate() {
            content.push_str(&format!("GPU{} usage: {}%\n", i + 1, usage));
        }
    }

    // Disk usage
    for (name, usage) in disk_usage() {
        content.push_str(&format!("Disk {} usage: {}%\n", name, usage));
    }

    // Network usage (only the first entry)
    if let Some((receive, transmit)) = network_usage().get(0) {
        content.push_str(&format!(
            "Network receive: {:.2} Kbps, Network transmit: {:.2} Kbps\n",
            (*receive as f64) / 1000.0,
            (*transmit as f64) / 1000.0
        ));
    }

    // Append to log file
    let log_path = "Log_file.txt";
    let mut file = if Path::new(log_path).exists() {
        OpenOptions::new().append(true).open(log_path).unwrap()
    } else {
        File::create(log_path).unwrap()
    };

    // Write content to file
    if let Err(e) = file.write_all(content.as_bytes()) {
        eprintln!("Failed to write to log file: {}", e);
    }
}