//! Integration tests for merix-resources — System resource management and hardware abstraction

use merix_resources::ResourceManager;

#[tokio::test]
async fn integration_test_resource_manager_full_hardware_detection() {
    let rm = ResourceManager::new().expect("Failed to create ResourceManager");

    // CPU detection (always present)
    let cores = rm.cpu_cores();
    println!("Detected logical CPU cores: {}", cores);
    assert!(cores > 0, "At least one CPU core must be detected");

    // Memory pressure (always present)
    let pressure = rm.memory_pressure_percent();
    println!("Current memory pressure: {}%", pressure);
    assert!(pressure <= 100, "Memory pressure must be 0-100%");

    // GPU / VRAM (NVML optional — must not panic)
    if let Some(vram_mb) = rm.available_vram_mb() {
        println!("Available VRAM: {} MB (NVIDIA GPU detected)", vram_mb);
        assert!(vram_mb > 0, "VRAM should be reported when GPU is present");
    } else {
        println!("[INFO] No NVIDIA GPU detected or NVML unavailable — this is expected on non-GPU systems");
    }

    println!("ResourceManager full hardware detection lifecycle completed successfully");
}

#[tokio::test]
async fn integration_test_resource_manager_cuda_aware_behavior() {
    let rm = ResourceManager::new().expect("Failed to create ResourceManager");

    // This test ensures the ResourceManager is ready for llama crate's CUDA path
    let pressure = rm.memory_pressure_percent();
    let vram = rm.available_vram_mb();

    println!(
        "CUDA-aware resource snapshot — memory_pressure: {}%, vram_mb: {:?}",
        pressure, vram
    );

    // No hard assertions on VRAM (GPU may be offline in CI), but the call must succeed
    assert!(pressure <= 100);
}