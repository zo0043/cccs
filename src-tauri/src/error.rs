// CCCS Error handling
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Claude Code installation not found")]
    ClaudeNotFound,
    
    #[error("Configuration file error: {0}")]
    ConfigError(String),
    
    #[error("Tray operation failed: {0}")]
    TrayError(String),
    
    #[error("File system error: {0}")]
    FileSystemError(String),
    
    #[error("Permission denied: {0}")]
    PermissionError(String),
    
    #[error("Settings error: {0}")]
    SettingsError(String),
    
    #[error("Monitor service error: {0}")]
    MonitorError(String),
    
    #[error("I18n error: {0}")]
    I18nError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Tauri error: {0}")]
    TauriError(#[from] tauri::Error),
}

// Convenience type alias
pub type AppResult<T> = Result<T, AppError>;