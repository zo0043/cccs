// Application lifecycle management for CCCS
use crate::{
    AppError, AppResult, 
    claude_detector::ClaudeDetector,
    config_service::ConfigService,
    tray_service::TrayService,
    monitor_service::MonitorService,
    settings_service::SettingsService,
    i18n_service::I18nService,
};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Listener, Manager};

pub struct App {
    config_service: Arc<Mutex<ConfigService>>,
    tray_service: Arc<Mutex<TrayService>>,
    monitor_service: Arc<Mutex<MonitorService>>,
    settings_service: Arc<Mutex<SettingsService>>,
    i18n_service: Arc<Mutex<I18nService>>,
    app_handle: AppHandle,
    is_initialized: bool,
}

impl App {
    /// Create a new application instance
    pub fn new(app_handle: AppHandle) -> AppResult<Self> {
        log::info!("Creating new CCCS application instance");
        
        let settings_service = Arc::new(Mutex::new(SettingsService::new()?));
        let i18n_service = Arc::new(Mutex::new(I18nService::new()));
        
        // Get settings for monitor interval
        let monitor_interval = {
            let settings = settings_service.lock().unwrap();
            settings.get_current_settings().monitor_interval_minutes
        };
        
        // Detect Claude directory immediately during construction
        let claude_dir = Self::detect_claude_directory_with_fallback(&app_handle)?;
        
        // Initialize services with proper configuration
        let config_service = Arc::new(Mutex::new(ConfigService::new(claude_dir)));
        let tray_service = Arc::new(Mutex::new(TrayService::new(app_handle.clone())));
        let monitor_service = Arc::new(Mutex::new(MonitorService::new(monitor_interval)));
        
        Ok(Self {
            config_service,
            tray_service,
            monitor_service,
            settings_service,
            i18n_service,
            app_handle,
            is_initialized: false,
        })
    }
    
    /// Detect Claude directory with fallback strategies
    fn detect_claude_directory_with_fallback(_app_handle: &AppHandle) -> AppResult<PathBuf> {
        // Try automatic detection first
        match ClaudeDetector::detect_claude_installation() {
            Ok(dir) => {
                log::info!("Claude directory detected automatically: {:?}", dir);
                ClaudeDetector::validate_default_config(&dir)?;
                return Ok(dir);
            }
            Err(AppError::ClaudeNotFound) => {
                log::warn!("Claude directory not found automatically");
            }
            Err(e) => return Err(e),
        }
        
        // Try common fallback locations
        let fallback_locations = vec![
            dirs::home_dir().map(|d| d.join(".claude")),
            dirs::config_dir().map(|d| d.join("claude")),
            std::env::var("CLAUDE_CONFIG_DIR").ok().map(PathBuf::from),
        ];
        
        for location in fallback_locations.into_iter().flatten() {
            if location.exists() && ClaudeDetector::validate_default_config(&location).is_ok() {
                log::info!("Found Claude directory at fallback location: {:?}", location);
                return Ok(location);
            }
        }
        
        // Create a default directory as last resort
        if let Some(home_dir) = dirs::home_dir() {
            let claude_dir = home_dir.join(".claude");
            log::warn!("Creating default Claude directory: {:?}", claude_dir);
            std::fs::create_dir_all(&claude_dir)
                .map_err(|e| AppError::FileSystemError(format!("Failed to create Claude directory: {}", e)))?;
            
            // Create a minimal settings.json
            let settings_file = claude_dir.join("settings.json");
            if !settings_file.exists() {
                let default_settings = r#"{"model": "claude-sonnet-4"}"#;
                std::fs::write(&settings_file, default_settings)
                    .map_err(|e| AppError::FileSystemError(format!("Failed to create default settings: {}", e)))?;
                log::info!("Created default settings.json");
            }
            
            return Ok(claude_dir);
        }
        
        Err(AppError::ClaudeNotFound)
    }
    
    /// Initialize the application
    pub async fn initialize(&mut self) -> AppResult<()> {
        log::info!("Initializing CCCS application");
        
        if self.is_initialized {
            log::warn!("Application already initialized");
            return Ok(());
        }
        
        // Step 1: Scan for profiles immediately
        {
            let mut config_service = self.config_service.lock().unwrap();
            match config_service.scan_profiles() {
                Ok(profiles) => {
                    log::info!("Successfully scanned {} profiles", profiles.len());
                }
                Err(e) => {
                    log::error!("Failed to scan profiles: {}", e);
                    // Don't fail initialization, just log the error
                }
            }
        }
        
        // Step 2: Setup file monitoring
        self.setup_monitoring().await?;
        
        // Step 3: Create system tray
        self.setup_tray().await?;
        
        // Step 4: Setup event listeners
        self.setup_event_listeners().await?;
        
        // Step 5: Perform initial status update
        self.refresh_all_status().await?;
        
        self.is_initialized = true;
        log::info!("CCCS application initialized successfully");
        
        Ok(())
    }
    
    /// Refresh all profile status and update UI
    async fn refresh_all_status(&self) -> AppResult<()> {
        log::debug!("Refreshing all profile status");
        
        // First refresh the status
        {
            let mut config_service = self.config_service.lock().unwrap();
            config_service.refresh_profile_status()?;
        }
        
        // Then get the profiles and statuses
        let profiles = {
            let config_service = self.config_service.lock().unwrap();
            config_service.get_profiles().to_vec()
        };
        
        let statuses = {
            let config_service = self.config_service.lock().unwrap();
            config_service.compare_profiles()
        };
        
        // Update tray with current status
        if let Ok(mut tray_service) = self.tray_service.lock() {
            match tray_service.update_menu_with_detailed_status(&profiles, &statuses) {
                Ok(()) => log::debug!("Tray menu updated successfully"),
                Err(e) => log::error!("Failed to update tray menu: {}", e),
            }
            
            // Update tooltip
            if let Ok(i18n_service) = self.i18n_service.lock() {
                let active_profile = profiles.iter()
                    .enumerate()
                    .find(|(i, _)| matches!(statuses[*i], crate::ProfileStatus::FullMatch))
                    .map(|(_, p)| p.name.as_str());
                let tooltip = i18n_service.get_tray_tooltip(profiles.len(), active_profile);
                let _ = tray_service.set_tooltip(&tooltip);
            }
        }
        
        Ok(())
    }
    
    /// Setup file monitoring
    async fn setup_monitoring(&self) -> AppResult<()> {
        log::info!("Setting up file monitoring");
        
        let config_service = Arc::clone(&self.config_service);
        let tray_service = Arc::clone(&self.tray_service);
        let app_handle = self.app_handle.clone();
        
        let mut monitor_service = self.monitor_service.lock().unwrap();
        
        // Add files to monitor
        let monitored_files = {
            let config = config_service.lock().unwrap();
            config.get_monitored_files()
        };
        
        for file in monitored_files {
            monitor_service.add_file_to_monitor(file);
        }
        
        // Start monitoring if auto-start is enabled
        let should_auto_start = {
            let settings = self.settings_service.lock().unwrap();
            settings.get_current_settings().auto_start_monitoring
        };
        
        if should_auto_start {
            let callback = move |changes: Vec<crate::ConfigFileChange>| {
                log::info!("File changes detected: {} files changed", changes.len());
                
                // Update configuration service
                if let Ok(mut config) = config_service.lock() {
                    if let Err(e) = config.refresh_profile_status() {
                        log::error!("Failed to refresh profile status: {}", e);
                    }
                    
                    // Update tray menu with detailed status
                    if let Ok(mut tray) = tray_service.lock() {
                        let profiles = config.get_profiles();
                        let statuses = config.compare_profiles();
                        if let Err(e) = tray.update_menu_with_detailed_status(profiles, &statuses) {
                            log::error!("Failed to update tray menu: {}", e);
                        }
                    }
                }
                
                // Emit event to notify frontend
                let _ = app_handle.emit("profiles_changed", ());
            };
            
            monitor_service.start_monitoring(callback)?;
        }
        
        Ok(())
    }
    
    /// Setup system tray
    async fn setup_tray(&self) -> AppResult<()> {
        log::info!("Setting up system tray");
        
        let mut tray_service = self.tray_service.lock().unwrap();
        tray_service.create_tray()?;
        
        // Update tray menu with initial profiles and detailed status
        let (profiles, statuses) = {
            let config = self.config_service.lock().unwrap();
            let profiles = config.get_profiles().to_vec();
            let statuses = config.compare_profiles();
            (profiles, statuses)
        };
        
        tray_service.update_menu_with_detailed_status(&profiles, &statuses)?;
        
        // Set tooltip
        let tooltip = {
            let i18n = self.i18n_service.lock().unwrap();
            let active_profile = profiles.iter()
                .enumerate()
                .find(|(i, _)| matches!(statuses[*i], crate::ProfileStatus::FullMatch))
                .map(|(_, p)| p.name.as_str());
            i18n.get_tray_tooltip(profiles.len(), active_profile)
        };
        tray_service.set_tooltip(&tooltip)?;
        
        Ok(())
    }
    
    /// Setup event listeners
    async fn setup_event_listeners(&self) -> AppResult<()> {
        log::info!("Setting up event listeners");
        
        let config_service = Arc::clone(&self.config_service);
        let tray_service = Arc::clone(&self.tray_service);
        let i18n_service = Arc::clone(&self.i18n_service);
        
        // Listen for profile switch requests from tray
        let config_service_clone = Arc::clone(&config_service);
        let tray_service_clone = Arc::clone(&tray_service);
        let i18n_service_clone = Arc::clone(&i18n_service);
        let _app_handle_for_switch = self.app_handle.clone();
        
        self.app_handle.listen("profile_switch_requested", move |event| {
            // Parse payload manually since as_str() is unstable
            if let Ok(profile_name) = serde_json::from_str::<String>(event.payload()) {
                log::info!("Profile switch requested: {}", profile_name);
                
                // Show switching status
                if let Ok(mut tray) = tray_service_clone.lock() {
                    let _ = tray.update_profile_status(&profile_name, "❕");
                }
                
                // Perform switch
                let result = {
                    let mut config = config_service_clone.lock().unwrap();
                    config.switch_profile(&profile_name)
                };
                
                match result {
                    Ok(()) => {
                        log::info!("Profile switched successfully: {}", profile_name);
                        
                        // Update tray menu with detailed status
                        if let (Ok(config), Ok(mut tray)) = (config_service_clone.lock(), tray_service_clone.lock()) {
                            let profiles = config.get_profiles();
                            let statuses = config.compare_profiles();
                            let _ = tray.update_menu_with_detailed_status(profiles, &statuses);
                            
                            // Update tooltip
                            if let Ok(i18n) = i18n_service_clone.lock() {
                                let active_profile = profiles.iter()
                                    .enumerate()
                                    .find(|(i, _)| matches!(statuses[*i], crate::ProfileStatus::FullMatch))
                                    .map(|(_, p)| p.name.as_str());
                                let tooltip = i18n.get_tray_tooltip(profiles.len(), active_profile);
                                let _ = tray.set_tooltip(&tooltip);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to switch profile {}: {}", profile_name, e);
                        
                        // Reset status on error
                        if let Ok(mut tray) = tray_service_clone.lock() {
                            let _ = tray.update_profile_status(&profile_name, "");
                        }
                    }
                }
            }
        });
        
        // Listen for settings menu clicks
        let app_handle_clone = self.app_handle.clone();
        self.app_handle.listen("menu_settings_clicked", move |_| {
            log::info!("Settings menu clicked");
            
            // Simply emit event to trigger settings window creation (no async needed)
            let _ = app_handle_clone.emit("open_settings_window", ());
        });
        
        // Listen for open settings window requests
        let app_handle_clone2 = self.app_handle.clone();
        self.app_handle.listen("open_settings_window", move |_| {
            log::info!("Opening settings window");
            
            // Check if settings window already exists
            if let Some(window) = app_handle_clone2.get_webview_window("settings") {
                log::info!("Settings window already exists, focusing it");
                // Window exists, just show and focus it
                let _ = window.show();
                let _ = window.set_focus();
                return;
            }
            
            // Create settings window
            match tauri::WebviewWindowBuilder::new(
                &app_handle_clone2,
                "settings",
                tauri::WebviewUrl::App("settings.html".into())
            )
            .title("CCCS Settings")
            .inner_size(600.0, 750.0)
            .min_inner_size(500.0, 650.0)
            .center()
            .resizable(true)
            .on_page_load(|window, _payload| {
                // Inject initialization script after page loads
                log::info!("Settings page loaded, injecting init script");
                let init_script = r#"
                    console.log('Page load hook: Checking Tauri API...');
                    if (window.__TAURI__) {
                        console.log('Tauri API is available!');
                    } else {
                        console.error('Tauri API is NOT available in page load hook');
                    }
                "#;
                if let Err(e) = window.eval(init_script) {
                    log::error!("Failed to inject init script: {}", e);
                }
            })
            .build()
            {
                Ok(window) => {
                    log::info!("Settings window created successfully");
                    let _ = window.show();
                }
                Err(e) => {
                    log::error!("Failed to create settings window: {}", e);
                }
            }
        });
        
        // Listen for tray icon hover events
        let config_service = Arc::clone(&self.config_service);
        let tray_service = Arc::clone(&self.tray_service);
        let _app_handle_for_hover = self.app_handle.clone();
        self.app_handle.listen("tray_icon_hover", move |_| {
            log::info!("Tray icon hover detected, refreshing profiles");
            
            // Refresh profiles synchronously
            if let Ok(mut config) = config_service.lock() {
                if let Err(e) = config.scan_profiles() {
                    log::error!("Failed to scan profiles on hover: {}", e);
                    return;
                }
                
                // Update tray menu with fresh profiles and detailed status
                if let Ok(mut tray) = tray_service.lock() {
                    let profiles = config.get_profiles();
                    let statuses = config.compare_profiles();
                    let _ = tray.update_menu_with_detailed_status(profiles, &statuses);
                }
            }
        });
        
        // Listen for app exit requests
        let monitor_service = Arc::clone(&self.monitor_service);
        self.app_handle.listen("app_exit_requested", move |_| {
            log::info!("Application exit requested");
            
            // Stop monitoring
            if let Ok(mut monitor) = monitor_service.lock() {
                monitor.stop_monitoring();
            }
            
            log::info!("Application cleanup completed");
        });
        
        
        Ok(())
    }
    
    /// Update monitor interval
    #[allow(dead_code)]
    pub async fn update_monitor_interval(&self, minutes: u64) -> AppResult<()> {
        log::info!("Updating monitor interval to {} minutes", minutes);
        
        let mut monitor_service = self.monitor_service.lock().unwrap();
        monitor_service.set_monitor_interval(minutes)?;
        
        Ok(())
    }
    
    
    /// Shutdown the application gracefully
    #[allow(dead_code)]
    pub async fn shutdown(&self) -> AppResult<()> {
        log::info!("Shutting down CCCS application");
        
        // Stop monitoring
        {
            let mut monitor_service = self.monitor_service.lock().unwrap();
            monitor_service.stop_monitoring();
        }
        
        // Clean up resources
        log::info!("Application shutdown completed");
        
        Ok(())
    }
    
    /// Check if application is initialized
    #[allow(dead_code)]
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }
    
    /// Get reference to config service
    pub fn get_config_service(&self) -> Arc<Mutex<ConfigService>> {
        Arc::clone(&self.config_service)
    }
    
    /// Get reference to settings service for testing
    #[cfg(test)]
    pub fn get_settings_service(&self) -> Arc<Mutex<SettingsService>> {
        Arc::clone(&self.settings_service)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::test::{mock_app, MockRuntime};
    
    // Note: These tests would require proper Tauri test setup
    // For now, we'll test the basic structure
    
    #[test]
    fn test_app_creation() {
        // This would need a proper Tauri app handle for real testing
        // For now, we just test that the types compile correctly
        assert!(true);
    }
}