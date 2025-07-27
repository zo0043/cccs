// Performance tests for CCCS components
// This module is only available in test builds

#[cfg(test)]
pub mod tests {
    use crate::{AppResult, PerformanceTestConfig};
    use std::time::{Duration, Instant};
    use std::path::PathBuf;
    use std::fs;
    use tempfile::TempDir;
    
    pub struct PerformanceTestSuite {
        temp_dir: TempDir,
        test_files: Vec<PathBuf>,
    }
    
    impl PerformanceTestSuite {
        pub fn new() -> AppResult<Self> {
            let temp_dir = TempDir::new()
                .map_err(|e| crate::AppError::FileSystemError(format!("Failed to create temp directory: {}", e)))?;
            
            Ok(Self {
                temp_dir,
                test_files: Vec::new(),
            })
        }
        
        pub fn setup_test_files(&mut self, config: &PerformanceTestConfig) -> AppResult<()> {
            let test_content = "x".repeat(config.file_size_bytes);
            
            for i in 0..config.file_count {
                let file_path = self.temp_dir.path().join(format!("test_{}.settings.json", i));
                let json_content = format!(r#"{{"test_file": {}, "content": "{}"}}"#, i, test_content);
                
                fs::write(&file_path, json_content)
                    .map_err(|e| crate::AppError::FileSystemError(format!("Failed to create test file: {}", e)))?;
                
                self.test_files.push(file_path);
            }
            
            Ok(())
        }
        
        pub fn get_test_dir(&self) -> &std::path::Path {
            self.temp_dir.path()
        }
        
        pub fn get_test_files(&self) -> &[PathBuf] {
            &self.test_files
        }
    }

    #[derive(Debug, Clone)]
    pub struct PerformanceTestResult {
        pub test_name: String,
        pub total_duration: Duration,
        pub operations_count: u64,
        pub success: bool,
        pub error_message: Option<String>,
    }
    
    impl PerformanceTestResult {
        pub fn print_summary(&self) {
            println!("=== {} ===", self.test_name);
            println!("Success: {}", self.success);
            if let Some(ref error) = self.error_message {
                println!("Error: {}", error);
            }
            println!("Total Duration: {:?}", self.total_duration);
            println!("Operations: {}", self.operations_count);
            println!();
        }
    }
}