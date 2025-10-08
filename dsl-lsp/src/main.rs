use clap::{Parser, Subcommand};
use std::net::SocketAddr;

mod websocket_server;

#[derive(Parser)]
#[command(name = "dsl-lsp-server")]
#[command(about = "DSL Language Server with AI Agent support", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Port to listen on for TCP connections
    #[arg(short, long, default_value = "3030")]
    port: u16,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the language server using stdio (for IDE integration)
    Stdio,

    /// Run the language server using TCP (for remote connections)
    Tcp {
        /// Address to bind to
        #[arg(short, long, default_value = "127.0.0.1")]
        address: String,
    },

    /// Run the language server using WebSocket (for web-based IDEs)
    Websocket {
        /// Address to bind to
        #[arg(short, long, default_value = "127.0.0.1")]
        address: String,
    },

    /// Generate data dictionary from sample KYC data
    GenerateDict {
        /// Output path for the data dictionary
        #[arg(short, long, default_value = "./data_dictionary")]
        output: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    match cli.command {
        None | Some(Commands::Stdio) => {
            log::info!("Starting DSL Language Server in stdio mode...");
            dsl_lsp::run_server().await;
        }

        Some(Commands::Tcp { address }) => {
            let addr: SocketAddr = format!("{}:{}", address, cli.port).parse()?;
            log::info!("Starting DSL Language Server on TCP {}", addr);

            run_tcp_server(addr).await?;
        }

        Some(Commands::Websocket { address }) => {
            let addr: SocketAddr = format!("{}:{}", address, cli.port).parse()?;
            log::info!("Starting DSL Language Server on WebSocket {}", addr);

            websocket_server::run_websocket_server(addr).await?;
        }

        Some(Commands::GenerateDict { output }) => {
            log::info!("Generating data dictionary to {}", output);
            generate_data_dictionary(&output)?;
        }
    }

    Ok(())
}

async fn run_tcp_server(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::net::TcpListener;
    use tower_lsp::{LspService, Server};

    let listener = TcpListener::bind(addr).await?;
    log::info!("TCP server listening on {}", addr);

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        log::info!("New TCP connection from {}", peer_addr);

        let (read_stream, write_stream) = tokio::io::split(stream);

        tokio::spawn(async move {
            let (service, socket) = LspService::new(|client| dsl_lsp::Backend::new(client));
            Server::new(read_stream, write_stream, socket)
                .serve(service)
                .await;
        });
    }
}

fn generate_data_dictionary(output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    use dsl_lsp::data_dictionary::DataDictionary;
    use std::fs;

    // Create output directory
    fs::create_dir_all(output_path)?;

    // Generate default KYC dictionary
    let dict = DataDictionary::create_default_kyc_dictionary();

    // Save entities
    let entities_json = serde_json::to_string_pretty(&dict.entities)?;
    fs::write(format!("{}/entities.json", output_path), entities_json)?;
    log::info!("Generated entities.json");

    // Save domains
    let domains_json = serde_json::to_string_pretty(&dict.domains)?;
    fs::write(format!("{}/domains.json", output_path), domains_json)?;
    log::info!("Generated domains.json");

    // Save lookups
    let lookups_json = serde_json::to_string_pretty(&dict.lookups)?;
    fs::write(format!("{}/lookups.json", output_path), lookups_json)?;
    log::info!("Generated lookups.json");

    // Save relationships
    let relationships_json = serde_json::to_string_pretty(&dict.relationships)?;
    fs::write(format!("{}/relationships.json", output_path), relationships_json)?;
    log::info!("Generated relationships.json");

    // Save complete dictionary
    let complete_json = serde_json::to_string_pretty(&dict)?;
    fs::write(format!("{}/complete.json", output_path), complete_json)?;
    log::info!("Generated complete.json");

    log::info!("Successfully generated data dictionary at {}", output_path);

    Ok(())
}