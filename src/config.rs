use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub files_root: PathBuf,
    pub log_level: String,
}

impl Config {
    pub fn new() -> Result<Self> {
        // Load .env file if present (for development)
        dotenvy::dotenv().ok();
        
        // Get FILES_ROOT from environment or use default
        let files_root = match std::env::var("FILES_ROOT") {
            Ok(path) => PathBuf::from(path),
            Err(_) => {
                eprintln!("FILES_ROOT not set, using current directory");
                std::env::current_dir()?
            }
        };
        
        // Verify the path exists
        if !files_root.exists() {
            anyhow::bail!("FILES_ROOT directory does not exist: {:?}", files_root);
        }
            
        let log_level = std::env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_string());
            
        Ok(Config {
            files_root,
            log_level,
        })
    }
}