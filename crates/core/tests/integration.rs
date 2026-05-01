use merix_core::CoreRuntime;
use merix_memory::{EtherealMemory, PersistentMemory};
use merix_schemas::TaskStatus;
use anyhow::Result;
use std::env;

#[tokio::test]
async fn integration_core_ethereal_memory_full_lifecycle() -> Result<()> {
    let memory = EtherealMemory::new();
    let core = CoreRuntime::new(memory).await?;

    let session = core.create_session(Some("Integration Ethereal Session".into()), None).await?;
    let task = core.create_task(session.id, "Integration test task".into()).await?;

    core.execute_task(task.id).await?;

    let loaded = core.load_task(task.id).await?.unwrap();
    assert_eq!(loaded.status, TaskStatus::Completed);

    println!("✅ Ethereal memory integration test passed");
    Ok(())
}

#[tokio::test]
async fn integration_core_persistent_memory_full_lifecycle() -> Result<()> {
    let db_path = env::temp_dir().join("merix_integration_test.db");
    let memory = PersistentMemory::new_at_path(&db_path).await?;
    let core = CoreRuntime::new(memory).await?;

    let session = core.create_session(Some("Integration Persistent Session".into()), None).await?;
    let task = core.create_task(session.id, "Persistent integration task".into()).await?;

    core.execute_task(task.id).await?;

    let loaded = core.load_task(task.id).await?.unwrap();
    assert_eq!(loaded.status, TaskStatus::Completed);

    println!("✅ Persistent memory integration test passed");
    Ok(())
}

#[tokio::test]
async fn integration_core_prevents_concurrent_execution() -> Result<()> {
    let memory = EtherealMemory::new();
    let core = CoreRuntime::new(memory).await?;

    let session = core.create_session(Some("Concurrency Test".into()), None).await?;
    let task = core.create_task(session.id, "Concurrency task".into()).await?;

    core.execute_task(task.id).await?;

    // Second execution must be rejected
    let result = core.execute_task(task.id).await;
    assert!(result.is_err(), "Expected error on double execution");

    println!("✅ Concurrency prevention test passed");
    Ok(())
}
