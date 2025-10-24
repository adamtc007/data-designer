//! CBU DSL Language Server Binary
//! Standalone LSP server for the CBU DSL

use tower_lsp::{LspService, Server};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create LSP service
    let (service, socket) = cbu_dsl_lsp::create_lsp_service();

    // Start the server
    tracing::info!("Starting CBU DSL Language Server...");
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service)
        .await;
}