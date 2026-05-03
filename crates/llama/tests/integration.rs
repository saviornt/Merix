use merix_llama::LlamaRuntime;
use std::env;

#[tokio::test]
async fn integration_test_llama_runtime_full_lifecycle_with_embedding_hotswap() {
    // User-provided test models
    let llm_path = env::var("MERIX_TEST_LLM_MODEL")
        .unwrap_or_else(|_| "tests/test_models/tinyllama-1.1b-chat.gguf".to_string());

    let embed_path = env::var("MERIX_TEST_EMBED_MODEL")
        .unwrap_or_else(|_| "tests/test_models/nomic-embed-text-v1.5.gguf".to_string());

    if !std::path::Path::new(&llm_path).exists() {
        println!("[SKIP] LLM test model not found at {llm_path} — skipping integration test");
        return;
    }

    let mut runtime = LlamaRuntime::new().expect("Failed to create LlamaRuntime");

    println!("✅ LlamaRuntime initialized");

    // 1. Load main LLM
    runtime.load_model(&llm_path).expect("Failed to load LLM");
    println!("✅ LLM loaded: {}", llm_path);

    // 2. Hot-swap: offload LLM to system RAM
    runtime
        .offload_to_system_ram()
        .expect("Failed to offload LLM");
    println!("✅ LLM offloaded to system RAM");

    // 3. Load embedding model (while LLM is in RAM)
    if std::path::Path::new(&embed_path).exists() {
        runtime
            .load_model(&embed_path)
            .expect("Failed to load embedding model");
        println!("✅ Embedding model loaded: {}", embed_path);
        runtime.unload_model();
        println!("✅ Embedding model unloaded");
    } else {
        println!("[SKIP] Embedding model not found — hot-swap test partial");
    }

    // 4. Reload main LLM to VRAM
    runtime.load_model(&llm_path).expect("Failed to reload LLM");
    println!("✅ LLM reloaded to VRAM");

    runtime.unload_model();
    println!(
        "✅ Full lifecycle (LLM → offload → embedding → reload → unload) completed successfully"
    );
}
