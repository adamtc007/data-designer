// Standalone test for Leptos SSR integration
use app_lib::web_server_simple::{create_simple_server, App};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Leptos SSR integration...");

    // Create the Leptos server
    let app = create_simple_server().await?;

    // Bind to localhost
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await?;

    println!("🚀 Leptos SSR test server running on http://127.0.0.1:3001");
    println!("📝 Test the following:");
    println!("   • Visit http://127.0.0.1:3001 in browser");
    println!("   • Check server-side rendering");
    println!("   • Verify Monaco editor placeholder loads");
    println!("   • Test reactive button interactions");
    println!("\n🔧 Monaco Editor Integration Status:");
    println!("   ✅ SSR layout structure");
    println!("   ✅ Reactive state management");
    println!("   ⏳ Monaco initialization (client-side only)");
    println!("   ⏳ Full editor integration");

    // Run the server
    axum::serve(listener, app).await?;

    Ok(())
}