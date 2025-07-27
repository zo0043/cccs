// Claude Code installation detection
use crate::{AppError, AppResult};
use std::path::{Path, PathBuf};
use tauri::AppHandle;

pub struct ClaudeDetector;

impl ClaudeDetector {
    /// Detect Claude Code installation in the user's home directory
    pub fn detect_claude_installation() -> AppResult<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| AppError::ConfigError("Unable to find home directory".to_string()))?;
        
        let claude_dir = home_dir.join(".claude");
        
        if claude_dir.exists() && claude_dir.is_dir() {
            log::info!("Found Claude directory at: {:?}", claude_dir);
            Ok(claude_dir)
        } else {
            log::warn!("Claude directory not found at: {:?}", claude_dir);
            Err(AppError::ClaudeNotFound)
        }
    }
    
    /// Validate that the default settings.json file exists
    pub fn validate_default_config(claude_dir: &Path) -> AppResult<()> {
        let settings_file = claude_dir.join("settings.json");
        
        if settings_file.exists() && settings_file.is_file() {
            log::info!("Found default settings.json at: {:?}", settings_file);
            Ok(())
        } else {
            log::error!("Default settings.json not found at: {:?}", settings_file);
            Err(AppError::ConfigError(
                "Default settings.json file not found. Please run Claude Code at least once.".to_string()
            ))
        }
    }
    
    /// Show file picker dialog for manual Claude directory selection
    pub async fn show_directory_picker(app: &AppHandle) -> AppResult<Option<PathBuf>> {
        use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
        
        // First show a confirmation dialog
        let choice = app.dialog()
            .message("Claude Code installation not found. Would you like to manually select the Claude directory?")
            .kind(MessageDialogKind::Info)
            .blocking_show();
        
        if choice {
            // Show directory picker
            if let Some(folder_path) = app.dialog()
                .file()
                .set_title("Select Claude Code Directory")
                .blocking_pick_folder()
            {
                let claude_path = folder_path.as_path().unwrap();
                log::info!("User selected Claude directory: {:?}", claude_path);
                
                // Validate the selected directory
                Self::validate_default_config(claude_path)?;
                Ok(Some(claude_path.to_path_buf()))
            } else {
                log::info!("User cancelled directory selection");
                Ok(None)
            }
        } else {
            log::info!("User chose not to select directory manually");
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_validate_default_config_success() {
        let temp_dir = TempDir::new().unwrap();
        let settings_file = temp_dir.path().join("settings.json");
        fs::write(&settings_file, "{}").unwrap();
        
        let result = ClaudeDetector::validate_default_config(temp_dir.path());
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_default_config_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        
        let result = ClaudeDetector::validate_default_config(temp_dir.path());
        assert!(result.is_err());
        
        if let Err(AppError::ConfigError(msg)) = result {
            assert!(msg.contains("settings.json file not found"));
        } else {
            panic!("Expected ConfigError");
        }
    }
}