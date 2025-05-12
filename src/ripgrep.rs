use std::path::{Path, PathBuf};
use tokio::process::Command as TokioCommand;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, instrument};
use crate::error::AppError;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchOptions {
    /// Search pattern
    pub pattern: String,
    
    /// Relative path within the root directory
    #[serde(default)]
    pub path: String,
    
    /// Use fixed strings instead of regex (literal search)
    #[serde(default)]
    pub fixed_strings: bool,
    
    /// Case-sensitive search
    #[serde(default)]
    pub case_sensitive: bool,
    
    /// Include line numbers in output
    #[serde(default = "default_true")]
    pub line_numbers: bool,
    
    /// Number of context lines to show
    #[serde(default)]
    pub context_lines: Option<usize>,
    
    /// File types to include (e.g., "rust", "js")
    #[serde(default)]
    pub file_types: Vec<String>,
    
    /// Maximum depth to search
    #[serde(default)]
    pub max_depth: Option<usize>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchResult {
    pub matches: Vec<String>,
    pub stats: SearchStats,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchStats {
    pub matched_lines: usize,
    pub elapsed_ms: u64,
}

#[derive(Debug)]
pub struct RipgrepSearcher {
    root_dir: PathBuf,
}

impl RipgrepSearcher {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }
    
    /// Validate a search path to prevent path traversal attacks
    fn validate_path(&self, path: &str) -> Result<PathBuf, AppError> {
        let search_path = self.root_dir.join(path);
        
        // Canonicalize both paths to resolve any ".." components
        let canonical_search_path = match std::fs::canonicalize(&search_path) {
            Ok(p) => p,
            Err(_) => return Err(AppError::InvalidPath(path.to_string())),
        };
        
        let canonical_root = match std::fs::canonicalize(&self.root_dir) {
            Ok(p) => p,
            Err(_) => return Err(AppError::ConfigError("Could not resolve root directory".to_string())),
        };
        
        // Ensure the search path is within the root directory
        if !canonical_search_path.starts_with(&canonical_root) {
            return Err(AppError::PathTraversal(path.to_string()));
        }
        
        Ok(search_path)
    }
    
    #[instrument(skip(self, options), fields(pattern = %options.pattern))]
    pub async fn search(&self, options: &SearchOptions) -> Result<SearchResult, AppError> {
        debug!("Starting ripgrep search");
        
        // Build the search path
        let search_path = if options.path.is_empty() {
            self.root_dir.clone()
        } else {
            self.validate_path(&options.path)?
        };
        
        // Start timing the search
        let start = std::time::Instant::now();
        
        // Build the command
        let output = self.build_command(options, &search_path).await?;
        
        // Calculate elapsed time
        let elapsed = start.elapsed();
        
        // Parse the output
        let stdout = String::from_utf8(output.stdout)
            .map_err(|_| AppError::RipgrepError("Invalid UTF-8 in output".to_string()))?;
            
        let matches: Vec<String> = stdout
            .lines()
            .map(|s| s.to_string())
            .collect();
        
        // Create a copy of matches.len() before moving matches
        let matched_lines = matches.len();
        
        Ok(SearchResult {
            matches,
            stats: SearchStats {
                matched_lines,
                elapsed_ms: elapsed.as_millis() as u64,
            },
        })
    }
    
    async fn build_command(&self, options: &SearchOptions, search_path: &Path) -> Result<std::process::Output, AppError> {
        let mut cmd = TokioCommand::new("rg");
        
        // Configure output format
        cmd.arg("--no-config"); // Ignore user config files
        
        if options.fixed_strings {
            cmd.arg("-F"); // Fixed strings mode
        }
        
        if !options.case_sensitive {
            cmd.arg("-i"); // Case insensitive
        }
        
        if options.line_numbers {
            cmd.arg("-n"); // Line numbers
        }
        
        // Add context lines if specified
        if let Some(context) = options.context_lines {
            cmd.arg("-C").arg(context.to_string());
        }
        
        // Add file types if specified
        for file_type in &options.file_types {
            cmd.arg("-t").arg(file_type);
        }
        
        // Add max depth if specified
        if let Some(depth) = options.max_depth {
            cmd.arg("--max-depth").arg(depth.to_string());
        }
        
        // Add pattern and path
        cmd.arg(&options.pattern);
        cmd.arg(search_path);
        
        // Execute the command
        let output = cmd.output().await
            .map_err(|e| AppError::RipgrepError(format!("Failed to execute ripgrep: {}", e)))?;
            
        // Check if the command was successful
        // Note: ripgrep returns status code 1 when no matches found, which is not an error
        if !output.status.success() && output.status.code() != Some(1) {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!(%stderr, "Ripgrep command failed");
            return Err(AppError::RipgrepError(format!("Ripgrep failed: {}", stderr)));
        }
        
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;
    
    fn setup_test_files() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a test file
        let file_path = temp_dir.path().join("test_file.rs");
        let mut file = File::create(file_path).unwrap();
        writeln!(file, "fn hello_world() {{").unwrap();
        writeln!(file, "    println!(\"Hello, world!\");").unwrap();
        writeln!(file, "}}").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "fn search_function(query: &str) {{").unwrap();
        writeln!(file, "    println!(\"Searching for {{}}\", query);").unwrap();
        writeln!(file, "}}").unwrap();
        
        // Create another file with different content
        let file_path = temp_dir.path().join("test_file.js");
        let mut file = File::create(file_path).unwrap();
        writeln!(file, "function helloWorld() {{").unwrap();
        writeln!(file, "    console.log(\"Hello, world!\");").unwrap();
        writeln!(file, "}}").unwrap();
        
        temp_dir
    }
    
    #[tokio::test]
    async fn test_basic_search() {
        let temp_dir = setup_test_files();
        let searcher = RipgrepSearcher::new(temp_dir.path().to_path_buf());
        
        let options = SearchOptions {
            pattern: "hello".into(),
            path: "".into(),
            fixed_strings: true,
            case_sensitive: false,
            line_numbers: true,
            context_lines: None,
            file_types: vec![],
            max_depth: None,
        };
        
        let result = searcher.search(&options).await.unwrap();
        assert!(result.matches.len() >= 2); // Should find "hello" in both files
        
        // Test with file type filter
        let options = SearchOptions {
            pattern: "hello".into(),
            path: "".into(),
            fixed_strings: true,
            case_sensitive: false,
            line_numbers: true,
            context_lines: None,
            file_types: vec!["rs".into()],
            max_depth: None,
        };
        
        let result = searcher.search(&options).await.unwrap();
        assert_eq!(result.matches.len(), 1); // Should only find in Rust file
    }
    
    #[tokio::test]
    async fn test_path_traversal_prevention() {
        let temp_dir = setup_test_files();
        let searcher = RipgrepSearcher::new(temp_dir.path().to_path_buf());
        
        let options = SearchOptions {
            pattern: "hello".into(),
            path: "../../../etc/passwd".into(), // Attempt path traversal
            fixed_strings: true,
            case_sensitive: false,
            line_numbers: true,
            context_lines: None,
            file_types: vec![],
            max_depth: None,
        };
        
        let result = searcher.search(&options).await;
        assert!(result.is_err());
        match result {
            Err(AppError::PathTraversal(_)) => {}
            _ => panic!("Expected PathTraversal error"),
        }
    }
}