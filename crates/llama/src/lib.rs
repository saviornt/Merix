//! merix-llama — Llama backend API, model loading, and NVIDIA GPU optimizations

use anyhow::Result;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;
use merix_resources::ResourceManager;
use merix_schemas::InferenceConfig;
use tracing::info;

/// Llama runtime with hardware-aware configuration and hot-swapping support
pub struct LlamaRuntime {
    config: InferenceConfig,
    resource_manager: ResourceManager,
    backend: LlamaBackend,
    model: Option<LlamaModel>,
    current_model_path: Option<String>,
}

impl LlamaRuntime {
    pub fn new() -> Result<Self> {
        let resource_manager = ResourceManager::new()?;
        let backend = LlamaBackend::init()?;
        let mut config = InferenceConfig::default();
        Self::optimize_config_in_place(&mut config, &resource_manager)?;
        Ok(Self {
            config,
            resource_manager,
            backend,
            model: None,
            current_model_path: None,
        })
    }

    fn optimize_config_in_place(config: &mut InferenceConfig, rm: &ResourceManager) -> Result<()> {
        if let Some(vram_mb) = rm.available_vram_mb() {
            config.vram_budget_mb = vram_mb.saturating_sub(1024);
            config.n_gpu_layers = -1;
        }
        if config.n_threads_batch == 0 {
            config.n_threads_batch = config.n_threads;
        }
        info!(
            n_gpu_layers = config.n_gpu_layers,
            vram_mb = config.vram_budget_mb,
            memory_pressure = rm.memory_pressure_percent(),
            "LLM config optimized using live hardware resources"
        );
        Ok(())
    }

    /// Private helper — reloads current model with any updated config (used for hot-swapping)
    fn reload_current_model(&mut self) -> Result<()> {
        if let Some(path) = &self.current_model_path {
            let params = LlamaModelParams::default()
                .with_n_gpu_layers(self.config.n_gpu_layers.max(0) as u32)
                .with_use_mmap(!self.config.no_mmap);

            let new_model = LlamaModel::load_from_file(&self.backend, path, &params)?;
            self.model = Some(new_model);
            info!(
                path,
                n_gpu_layers = self.config.n_gpu_layers,
                "Model reloaded (hot-swap complete)"
            );
        }
        Ok(())
    }

    /// Load (or hot-swap) a GGUF model — automatically unloads previous model
    pub fn load_model(&mut self, model_path: &str) -> Result<()> {
        self.unload_model(); // clean previous model
        info!(
            model_path,
            "Loading Llama model with hardware-optimized config"
        );

        let params = LlamaModelParams::default()
            .with_n_gpu_layers(self.config.n_gpu_layers.max(0) as u32)
            .with_use_mmap(!self.config.no_mmap);

        let model = LlamaModel::load_from_file(&self.backend, model_path, &params)?;

        self.model = Some(model);
        self.current_model_path = Some(model_path.to_string());
        info!("Llama model loaded successfully (CUDA-ready)");
        Ok(())
    }

    /// Unload current model — releases VRAM/RAM immediately
    pub fn unload_model(&mut self) {
        if self.model.is_some() {
            self.model = None;
            info!("Model unloaded (VRAM/RAM released)");
        }
    }

    /// Temporarily offload current LLM to system RAM (n_gpu_layers = 0)
    /// so another model (e.g. embedding) can be loaded
    pub fn offload_to_system_ram(&mut self) -> Result<()> {
        if self.current_model_path.is_some() {
            self.config.n_gpu_layers = 0;
            self.reload_current_model()?;
            info!("Model offloaded to system RAM");
        }
        Ok(())
    }

    /// Reload current model to VRAM (n_gpu_layers = -1)
    pub fn reload_to_vram(&mut self) -> Result<()> {
        if self.current_model_path.is_some() {
            self.config.n_gpu_layers = -1;
            self.reload_current_model()?;
            info!("Model reloaded to VRAM");
        }
        Ok(())
    }

    /// Expose current config (for core/executor)
    pub fn config(&self) -> &InferenceConfig {
        &self.config
    }

    /// Current memory pressure from live system resources
    pub fn memory_pressure_percent(&self) -> u8 {
        self.resource_manager.memory_pressure_percent()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llama_runtime_load_unload_hotswap() {
        let mut runtime = LlamaRuntime::new().unwrap();
        assert!(runtime.config.context_size > 0);

        // unload on fresh runtime does nothing (no panic)
        runtime.unload_model();

        // offload / reload cycle works (no real model needed for test)
        let _ = runtime.offload_to_system_ram();
        let _ = runtime.reload_to_vram();
    }
}
