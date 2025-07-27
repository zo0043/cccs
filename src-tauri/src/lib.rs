// CCCS - Claude Code Configuration Switcher
// Core modules
mod app;
mod claude_detector;
mod config_service;
mod tray_service;
mod monitor_service;
mod settings_service;
mod i18n_service;
mod error;
mod types;

// Performance testing module (only in debug builds)
#[cfg(debug_assertions)]
pub mod performance_tests;

// Re-exports for public API
pub use error::AppError;
pub use types::*;
use config_service::ConfigService;
use claude_detector::ClaudeDetector;

pub type AppResult<T> = Result<T, AppError>;

use tauri::{AppHandle, Manager};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use app::App;

#[derive(Serialize)]
struct ProfilesInfo {
    claude_directory: String,
    profiles_count: usize,
    monitor_status: String,
}

#[tauri::command]
async fn get_profiles_info(app_state: tauri::State<'_, Arc<Mutex<App>>>) -> Result<ProfilesInfo, String> {
    log::info!("get_profiles_info called");
    
    let app = match app_state.try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to lock app state: {}", e);
            return Err("Failed to access application state".to_string());
        }
    };
    
    // Get config service from app
    let config_service = app.get_config_service();
    let config = match config_service.try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to lock config service: {}", e);
            return Err("Failed to access configuration service".to_string());
        }
    };
    
    let profiles = config.get_profiles();
    let claude_dir = config.get_claude_dir();
    
    log::info!("Returning profiles info: {} profiles found in {}", profiles.len(), claude_dir.display());
    
    Ok(ProfilesInfo {
        claude_directory: claude_dir.to_string_lossy().to_string(),
        profiles_count: profiles.len(),
        monitor_status: "inactive".to_string(),
    })
}

#[tauri::command]
async fn close_settings_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("settings") {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(setup_app)
        .invoke_handler(tauri::generate_handler![
            settings_service::get_settings,
            settings_service::update_monitor_interval,
            settings_service::update_auto_start_monitoring,
            settings_service::update_language,
            settings_service::update_show_notifications,
            settings_service::reset_settings_to_defaults,
            i18n_service::get_current_locale,
            i18n_service::set_locale,
            i18n_service::get_text,
            i18n_service::get_supported_locales,
            get_profiles_info,
            close_settings_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    if cfg!(debug_assertions) {
        log::info!("CCCS application starting in development mode");
    }

    let app_handle = app.handle().clone();

    // Initialize basic services with error handling
    match initialize_services(app) {
        Ok(()) => {
            log::info!("Services initialized successfully");
        }
        Err(e) => {
            log::error!("Failed to initialize services: {}", e);
            // Continue anyway - the app can still function without some services
        }
    }

    // Initialize CCCS app and store it in Tauri state
    // This needs to be done synchronously to ensure proper state management
    match initialize_cccs_app(app_handle.clone()) {
        Ok(cccs_app) => {
            log::info!("CCCS app created successfully");
            // Store the app instance
            app.manage(Arc::new(Mutex::new(cccs_app)));
            
            // Defer actual initialization to avoid blocking startup
            let app_handle_clone = app_handle.clone();
            std::thread::spawn(move || {
                // Add delay to ensure UI is ready
                std::thread::sleep(std::time::Duration::from_millis(2000));
                
                log::info!("Starting delayed CCCS initialization");
                
                // Get the app instance and initialize it
                if let Some(app_state) = app_handle_clone.try_state::<Arc<Mutex<App>>>() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut app = app_state.lock().unwrap();
                        match app.initialize().await {
                            Ok(()) => {
                                log::info!("CCCS application initialized successfully");
                            }
                            Err(e) => {
                                log::error!("Failed to initialize CCCS app: {}", e);
                            }
                        }
                    });
                }
            });
        }
        Err(e) => {
            log::error!("Failed to create CCCS app: {}", e);
            // App will continue to run in basic mode
        }
    }

    log::info!("CCCS setup completed");
    Ok(())
}

/// Initialize Tauri state services
fn initialize_services(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize settings service
    match settings_service::SettingsService::new() {
        Ok(settings_service) => {
            app.manage(std::sync::Mutex::new(settings_service));
            log::info!("Settings service initialized");
        }
        Err(e) => {
            log::error!("Failed to initialize settings service: {}", e);
            // Use default settings service
            let default_settings = settings_service::SettingsService::with_defaults();
            app.manage(std::sync::Mutex::new(default_settings));
        }
    }

    // Initialize i18n service
    let i18n_service = i18n_service::I18nService::new();
    app.manage(std::sync::Mutex::new(i18n_service));
    log::info!("I18n service initialized");

    // Note: Config service will be initialized as part of the App instance

    Ok(())
}

/// Create the main CCCS application
fn initialize_cccs_app(app_handle: AppHandle) -> Result<App, Box<dyn std::error::Error>> {
    log::info!("Creating CCCS application instance");

    // Create the main app with error handling
    let cccs_app = match app::App::new(app_handle.clone()) {
        Ok(app) => {
            log::info!("App instance created successfully");
            app
        }
        Err(e) => {
            log::error!("Failed to create App instance: {}", e);
            return Err(e.into());
        }
    };

    Ok(cccs_app)
}
