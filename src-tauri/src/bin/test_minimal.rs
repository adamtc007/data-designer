// Test minimal server standalone
use app_lib::web_server_minimal::create_minimal_server;

#[tokio::main]
async fn main() -> Result<(), String> {
    println!("🧪 Testing minimal server for Tauri integration...");

    let app = create_minimal_server().await?;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await
        .map_err(|e| format!("Failed to bind: {}", e))?;

    println!("🚀 Minimal server running on http://127.0.0.1:3001");
    println!("✅ Ready for Tauri integration!");

    axum::serve(listener, app).await
        .map_err(|e| format!("Server error: {}", e))?;
    Ok(())
}