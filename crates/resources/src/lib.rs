//! merix-resources — System resource management and hardware abstraction

use anyhow::Result;
use nvml_wrapper::Nvml;
use sysinfo::System;
use tracing::info;

/// Unified resource manager for CPU, RAM, and NVIDIA GPU (NVML)
pub struct ResourceManager {
    sys: System,
    nvml: Option<Nvml>,
}

impl ResourceManager {
    pub fn new() -> Result<Self> {
        let mut sys = System::new_all();
        sys.refresh_all();

        let nvml = match Nvml::init() {
            Ok(n) => {
                info!("NVML initialized — NVIDIA GPU detected");
                Some(n)
            }
            Err(_) => {
                info!("No NVIDIA GPU detected or NVML unavailable");
                None
            }
        };

        Ok(Self { sys, nvml })
    }

    /// Returns current memory pressure (0-100) for LLM context decisions
    pub fn memory_pressure_percent(&self) -> u8 {
        let used = self.sys.used_memory();
        let total = self.sys.total_memory();
        if total == 0 {
            0
        } else {
            ((used as f64 / total as f64) * 100.0) as u8
        }
    }

    /// Returns available VRAM in MB (NVIDIA only)
    pub fn available_vram_mb(&self) -> Option<u64> {
        self.nvml.as_ref().and_then(|n| {
            n.device_by_index(0).ok().map(|d| {
                d.memory_info()
                    .ok()
                    .map(|i| i.free / 1024 / 1024)
                    .unwrap_or(0)
            })
        })
    }

    /// Returns number of logical CPU cores
    pub fn cpu_cores(&self) -> usize {
        self.sys.cpus().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_manager_basic() {
        let rm = ResourceManager::new().unwrap();
        assert!(rm.cpu_cores() > 0);
        let _pressure = rm.memory_pressure_percent();
        // GPU optional
        let _vram = rm.available_vram_mb();
    }
}
