use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Ripgrep error: {0}")]
    RipgrepError(String),
    
    #[error("Path traversal attempt: {0}")]
    PathTraversal(String),
    
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("MCP error: {0}")]
    MCPError(String),
}