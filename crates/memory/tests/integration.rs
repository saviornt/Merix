use merix_memory::{EtherealMemory, MemoryLayer, PersistentMemory};
use merix_schemas::{Session, Task, Checkpoint, Skill, TaskStatus, Uuid};
use chrono::Utc;

#[tokio::test]
async fn test_memory_full_integration_roundtrip() {
    // 1. Create both memory layers
    let db_path = std::env::temp_dir().join("merix_integration_test.db");
    let persistent = PersistentMemory::new_at_path(db_path.as_path()).await.unwrap();
    persistent.init().await.unwrap();

    let ethereal = EtherealMemory::new();

    // 2. Create sample data
    let session_id = Uuid::new_v4();
    let task_id = Uuid::new_v4();
    let checkpoint_id = Uuid::new_v4();
    let skill_id = Uuid::new_v4();

    let session = Session {
        id: session_id,
        title: Some("Integration Test Session".to_string()),
        description: Some("Testing full memory roundtrip".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let task = Task {
        id: task_id,
        session_id,
        description: "Test task description".to_string(),
        status: TaskStatus::Running,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        parent_id: None,
    };

    let checkpoint = Checkpoint {
        id: checkpoint_id,
        task_id,
        sequence: 42,
        state: serde_json::json!({"key": "value", "progress": 75}),
        created_at: Utc::now(),
    };

    let skill = Skill {
        id: skill_id,
        name: "test_skill".to_string(),
        description: "A test skill for integration".to_string(),
        version: "1.0.0".to_string(),
        created_at: Utc::now(),
    };

    // 3. Store in Persistent
    persistent.store_session(session.clone()).await.unwrap();
    persistent.store_task(task.clone()).await.unwrap();
    persistent.store_checkpoint(checkpoint.clone()).await.unwrap();
    persistent.store_skill(skill.clone()).await.unwrap();

    // 4. Load from Persistent
    let loaded_session = persistent.load_session(session_id).await.unwrap().unwrap();
    let loaded_task = persistent.load_task(task_id).await.unwrap().unwrap();
    let loaded_checkpoint = persistent.load_checkpoint(checkpoint_id).await.unwrap().unwrap();
    let loaded_skill = persistent.load_skill(skill_id).await.unwrap().unwrap();

    // 5. Store into Ethereal
    ethereal.store_session(loaded_session.clone()).await.unwrap();
    ethereal.store_task(loaded_task.clone()).await.unwrap();
    ethereal.store_checkpoint(loaded_checkpoint.clone()).await.unwrap();
    ethereal.store_skill(loaded_skill.clone()).await.unwrap();

    // 6. Verify round-trip with field-by-field assertions (avoids SurrealDB RecordId deserialization issue)
    let roundtrip_session = ethereal.load_session(session_id).await.unwrap().unwrap();
    let roundtrip_task = ethereal.load_task(task_id).await.unwrap().unwrap();
    let roundtrip_checkpoint = ethereal.load_checkpoint(checkpoint_id).await.unwrap().unwrap();
    let roundtrip_skill = ethereal.load_skill(skill_id).await.unwrap().unwrap();

    assert_eq!(roundtrip_session.id, session.id);
    assert_eq!(roundtrip_session.title, session.title);
    assert_eq!(roundtrip_session.description, session.description);

    assert_eq!(roundtrip_task.id, task.id);
    assert_eq!(roundtrip_task.description, task.description);
    assert_eq!(roundtrip_task.status, task.status);

    assert_eq!(roundtrip_checkpoint.id, checkpoint.id);
    assert_eq!(roundtrip_checkpoint.sequence, checkpoint.sequence);
    assert_eq!(roundtrip_checkpoint.state, checkpoint.state);

    assert_eq!(roundtrip_skill.id, skill.id);
    assert_eq!(roundtrip_skill.name, skill.name);
    assert_eq!(roundtrip_skill.version, skill.version);

    println!("✅ Full MemoryLayer integration roundtrip test passed!");
}


