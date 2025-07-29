use sysinfo::System;
use once_cell::sync::Lazy;
use parking_lot::Mutex;

// Use a static Mutex to hold the System instance, initialized lazily.
static SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| {
    Mutex::new(System::new_all())
});

#[tauri::command]
pub fn get_cpu_usage() -> Result<f32, String> {
    let mut system = SYSTEM.lock();
    // Refresh CPU data. It's important to refresh before reading.
    system.refresh_cpu_usage();
    
    // Sum up the usage of all CPUs.
    let total_usage: f32 = system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum();
    let cpu_count = system.cpus().len() as f32;

    if cpu_count > 0.0 {
        Ok(total_usage / cpu_count)
    } else {
        Ok(0.0) // Avoid division by zero if no CPUs are detected
    }
}

#[tauri::command]
pub fn get_memory_usage() -> Result<f32, String> {
    let mut system = SYSTEM.lock();
    // Refresh memory data.
    system.refresh_memory();
    
    let total_memory = system.total_memory() as f32;
    let used_memory = (system.total_memory() - system.available_memory()) as f32;

    if total_memory > 0.0 {
        Ok((used_memory / total_memory) * 100.0)
    } else {
        Ok(0.0) // Avoid division by zero if total memory is 0
    }
}

// Placeholder command for online status
#[tauri::command]
pub fn get_online_status() -> Result<bool, String> {
    // TODO: Implement actual online status check (e.g., ping, network interface check)
    Ok(true) // Placeholder: always return true for now
}

// Function to start monitoring
pub fn start_monitoring(_app_handle: tauri::AppHandle) {
    // System monitoring is now passive - just respond to commands
    // The static SYSTEM instance will be updated when needed
}