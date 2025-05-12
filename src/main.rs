mod config;
mod error;
mod mcp;
mod ripgrep;

use anyhow::Result;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use tracing_subscriber::filter::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = config::Config::new()?;
    
    // Set up logging to stderr only, no color codes
    setup_logging(&config.log_level);
    
    // Stderr messages are fine as they won't interfere with JSON-RPC over stdout
    eprintln!("Starting ripgrep MCP server");
    eprintln!("Files root directory: {:?}", config.files_root);
    
    // Check if ripgrep is installed
    match which::which("rg") {
        Ok(path) => eprintln!("Found ripgrep at {:?}", path),
        Err(_) => {
            eprintln!("Error: ripgrep (rg) is not installed or not in PATH");
            std::process::exit(1);
        }
    }
    
    // Create and run the MCP server (will communicate over stdin/stdout)
    let server = mcp::MCPServer::new(config);
    
    // Run the server and ensure all errors go to stderr, not stdout
    if let Err(e) = server.run().await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
    
    // This should never be reached in normal operation
    eprintln!("Server shutdown");
    Ok(())
}

fn setup_logging(log_level: &str) {
    let level = match log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };
    
    // Create a custom subscriber that only logs to stderr
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("mcp_rg={}", level)));
        
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_ansi(false) // Disable ANSI color codes
        .with_writer(std::io::stderr) // Force all logging to stderr only
        .finish();
        
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");
}