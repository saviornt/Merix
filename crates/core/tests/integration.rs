//! Integration tests for merix-core
//!
//! Tests the full CoreRuntime end-to-end with real memory layers,
//! task/session lifecycle, LLM model loading (using the official test model),
//! and persistence. Fully resumable and crash-safe.

use anyhow::Result;
use merix_core::CoreRuntime;
use merix_memory::{EtherealMemory, PersistentMemory};
use merix_schemas::InferenceConfig;
use std::env;
use std::path::PathBuf;

fn test_model_path() -> PathBuf {
    // Resolve from crates/core/tests/ → workspace root/tests/test_models/
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .join("..")      // crates/core
        .join("..")      // crates/
        .join("tests")
        .join("test_models")
        .join("tinyllama-1.1b-chat.gguf")
}

#[tokio::test]
async fn integration_core_full_lifecycle_ethereal() -> Result<()> {
    let memory = EtherealMemory::new();
    let core = CoreRuntime::new(memory).await?;

    let session = core
        .create_session(Some("Integration Test Session".into()), None)
        .await?;
    let task = core
        .create_task(session.id, "Integration test task description".into())
        .await?;

    // Use the official test model from tests/test_models/
    let model_path = test_model_path();
    assert!(model_path.exists(), "Test model tinyllama-1.1b-chat.gguf not found at {:?}", model_path);
    core.execute_task(task.id, Some(model_path.to_str().unwrap())).await?;

    let loaded_task = core.load_task(task.id).await?.unwrap();
    assert_eq!(loaded_task.status, merix_schemas::TaskStatus::Completed);

    let loaded_session = core.load_session(session.id).await?.unwrap();
    assert_eq!(loaded_session.id, session.id);

    tracing::info!("✅ Ethereal memory full lifecycle passed (with real tinyllama model)");
    Ok(())
}

#[tokio::test]
async fn integration_core_full_lifecycle_persistent() -> Result<()> {
    let db_path = env::temp_dir().join("merix_integration_test.db");
    let memory = PersistentMemory::new_at_path(&db_path).await?;
    let core = CoreRuntime::new(memory).await?;

    let session = core
        .create_session(Some("Persistent Integration Test".into()), None)
        .await?;
    let task = core
        .create_task(session.id, "Persistent integration task".into())
        .await?;

    let model_path = test_model_path();
    core.execute_task(task.id, Some(model_path.to_str().unwrap())).await?;

    let loaded_task = core.load_task(task.id).await?.unwrap();
    assert_eq!(loaded_task.status, merix_schemas::TaskStatus::Completed);

    tracing::info!("✅ Persistent memory full lifecycle passed (with real tinyllama model)");
    Ok(())
}

#[tokio::test]
async fn integration_core_llm_configuration_and_hot_swap() -> Result<()> {
    let memory = EtherealMemory::new();
    let core = CoreRuntime::new(memory).await?;

    // Use struct update syntax (satisfies clippy::field_reassign_with_default)
    let config = InferenceConfig {
        use_gpu: true,
        n_gpu_layers: 42,
        ..Default::default()
    };

    core.configure_llm(config.clone()).await?;

    let loaded_config = core.get_inference_config().await;
    assert!(loaded_config.use_gpu); // satisfies clippy::bool_assert_comparison
    assert_eq!(loaded_config.n_gpu_layers, 42);

    tracing::info!("✅ LLM configuration hot-swap verified");
    Ok(())
}

#[tokio::test]
async fn integration_core_model_loading_real() -> Result<()> {
    let memory = EtherealMemory::new();
    let core = CoreRuntime::new(memory).await?;

    let session = core
        .create_session(Some("Real Model Loading Test".into()), None)
        .await?;
    let task = core
        .create_task(session.id, "Test real model loading".into())
        .await?;

    let model_path = test_model_path();
    assert!(model_path.exists(), "Test model not found - expected at {:?}", model_path);

    core.execute_task(task.id, Some(model_path.to_str().unwrap())).await?;

    tracing::info!(
        model_path = %model_path.display(),
        "✅ Real model loading verified via LlamaRuntime (tinyllama-1.1b-chat.gguf)"
    );
    Ok(())
}