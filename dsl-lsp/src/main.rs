use tower_lsp::{LspService, Server};
use dsl_lsp::Backend;

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend::new(client));

    Server::new(stdin, stdout, socket)
        .serve(service)
        .await;
}