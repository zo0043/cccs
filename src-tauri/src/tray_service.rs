// System tray service for CCCS
use crate::{AppError, AppResult, Profile};
use tauri::{
    menu::{Menu, MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter,
};

pub struct TrayService {
    app_handle: AppHandle,
    current_menu: Option<Menu<tauri::Wry>>,
    tray_id: String,
}

impl TrayService {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            current_menu: None,
            tray_id: "cccs_tray".to_string(),
        }
    }
    
    /// Create and initialize the system tray icon with enhanced error handling
    pub fn create_tray(&mut self) -> AppResult<()> {
        log::info!("Creating system tray icon");
        
        // Build initial menu with error handling
        let menu = match self.build_basic_menu() {
            Ok(menu) => menu,
            Err(e) => {
                log::error!("Failed to build basic menu: {}", e);
                // Create a minimal fallback menu
                self.build_fallback_menu()?
            }
        };
        
        self.current_menu = Some(menu.clone());
        
        // Create tray icon with error handling
        match self.create_tray_icon_safe(&menu) {
            Ok(_) => {
                log::info!("System tray icon created successfully");
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to create tray icon: {}", e);
                Err(e)
            }
        }
    }
    
    /// Create tray icon with safety checks
    fn create_tray_icon_safe(&self, menu: &Menu<tauri::Wry>) -> AppResult<()> {
        // Get icon with fallback
        let icon = match self.app_handle.default_window_icon() {
            Some(icon) => icon.clone(),
            None => {
                log::warn!("No default window icon found, creating tray without icon");
                return self.create_tray_without_icon(menu);
            }
        };
        
        // Try to create tray icon with specific ID
        let app_handle_clone = self.app_handle.clone();
        let _tray = TrayIconBuilder::with_id(&self.tray_id)
            .icon(icon)
            .menu(menu)
            .icon_as_template(true) // macOS: use template icon for dark/light mode
            .show_menu_on_left_click(false) // Only show menu on right-click
            .on_menu_event(move |app, event| {
                if let Err(e) = Self::handle_menu_event_safe(app, event) {
                    log::error!("Error handling menu event: {}", e);
                }
            })
            .on_tray_icon_event(move |_tray, event| {
                let app_handle = &app_handle_clone;
                if let Err(e) = Self::handle_tray_event_safe(app_handle, event) {
                    log::error!("Error handling tray event: {}", e);
                }
            })
            .build(&self.app_handle)
            .map_err(|e| AppError::TrayError(format!("Failed to build tray icon: {}", e)))?;
        
        Ok(())
    }
    
    /// Create tray without icon as fallback
    fn create_tray_without_icon(&self, menu: &Menu<tauri::Wry>) -> AppResult<()> {
        let app_handle_clone = self.app_handle.clone();
        let _tray = TrayIconBuilder::with_id(&self.tray_id)
            .menu(menu)
            .icon_as_template(true) // macOS: use template icon for dark/light mode
            .show_menu_on_left_click(false) // Only show menu on right-click
            .on_menu_event(move |app, event| {
                if let Err(e) = Self::handle_menu_event_safe(app, event) {
                    log::error!("Error handling menu event: {}", e);
                }
            })
            .on_tray_icon_event(move |_tray, event| {
                let app_handle = &app_handle_clone;
                if let Err(e) = Self::handle_tray_event_safe(app_handle, event) {
                    log::error!("Error handling tray event: {}", e);
                }
            })
            .build(&self.app_handle)
            .map_err(|e| AppError::TrayError(format!("Failed to build tray icon without icon: {}", e)))?;
        
        Ok(())
    }
    
    /// Create a minimal fallback menu
    fn build_fallback_menu(&self) -> AppResult<Menu<tauri::Wry>> {
        let menu = MenuBuilder::new(&self.app_handle)
            .text("settings", "Settings")
            .separator()
            .text("exit", "Exit")
            .build()
            .map_err(|e| AppError::TrayError(format!("Failed to build fallback menu: {}", e)))?;
        
        Ok(menu)
    }
    
    /// Build the basic menu structure (empty profiles, settings, exit)
    fn build_basic_menu(&self) -> AppResult<Menu<tauri::Wry>> {
        let menu = MenuBuilder::new(&self.app_handle)
            .separator()
            .item(&MenuItemBuilder::with_id("settings", "Settings").build(&self.app_handle)?)
            .item(&MenuItemBuilder::with_id("exit", "Exit").build(&self.app_handle)?)
            .build()?;
        
        Ok(menu)
    }
    
    /// Update menu with current profiles
    pub fn update_menu(&mut self, profiles: &[Profile]) -> AppResult<()> {
        log::info!("Updating tray menu with {} profiles", profiles.len());
        
        let mut menu_builder = MenuBuilder::new(&self.app_handle);
        
        // Add profile menu items
        for profile in profiles {
            let menu_text = if profile.is_active {
                format!("{} ✅", profile.name)
            } else {
                profile.name.clone()
            };
            
            let menu_item = MenuItemBuilder::with_id(
                format!("profile_{}", profile.name),
                menu_text
            ).build(&self.app_handle)?;
            
            menu_builder = menu_builder.item(&menu_item);
        }
        
        // Add separator and system menu items
        let menu = menu_builder
            .separator()
            .item(&MenuItemBuilder::with_id("settings", "Settings").build(&self.app_handle)?)
            .item(&MenuItemBuilder::with_id("exit", "Exit").build(&self.app_handle)?)
            .build()?;
        
        // Update the tray menu - get tray by ID
        if let Some(tray) = self.app_handle.tray_by_id(&self.tray_id) {
            tray.set_menu(Some(menu.clone()))?;
        }
        
        self.current_menu = Some(menu);
        log::info!("Tray menu updated successfully");
        
        Ok(())
    }
    
    /// Handle menu item click events with error handling
    fn handle_menu_event_safe(app: &AppHandle, event: tauri::menu::MenuEvent) -> AppResult<()> {
        let event_id = event.id().as_ref();
        log::info!("Menu item clicked: {}", event_id);
        
        match event_id {
            "settings" => {
                Self::handle_settings_click(app)
            }
            "exit" => {
                Self::handle_exit_click(app)
            }
            id if id.starts_with("profile_") => {
                let profile_name = id.strip_prefix("profile_").unwrap_or("");
                Self::handle_profile_click(app, profile_name)
            }
            _ => {
                log::warn!("Unhandled menu event: {}", event_id);
                Ok(())
            }
        }
    }
    
    /// Handle tray icon click events with error handling
    fn handle_tray_event_safe(app_handle: &AppHandle, event: TrayIconEvent) -> AppResult<()> {
        match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => {
                log::debug!("Tray icon left-clicked");
                // Could show/hide main window or show profiles menu
                Ok(())
            }
            TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => {
                log::debug!("Tray icon double-clicked");
                // Could open settings or main window
                Ok(())
            }
            TrayIconEvent::Enter { .. } => {
                log::debug!("Mouse entered tray icon area");
                // Trigger profile scan when mouse enters
                app_handle.emit("tray_icon_hover", ())
                    .map_err(|e| AppError::TrayError(format!("Failed to emit hover event: {}", e)))?;
                Ok(())
            }
            TrayIconEvent::Click {
                button: MouseButton::Right,
                button_state: MouseButtonState::Up,
                ..
            } => {
                log::debug!("Tray icon right-clicked");
                // Right click will show menu - trigger profile scan first
                app_handle.emit("tray_icon_hover", ())
                    .map_err(|e| AppError::TrayError(format!("Failed to emit hover event: {}", e)))?;
                Ok(())
            }
            _ => {
                log::debug!("Other tray event: {:?}", event);
                Ok(())
            }
        }
    }
    
    /// Handle settings menu item click
    fn handle_settings_click(app: &AppHandle) -> AppResult<()> {
        log::info!("Settings menu clicked");
        
        // Emit event to notify other parts of the application
        app.emit("menu_settings_clicked", ())
            .map_err(|e| AppError::TrayError(format!("Failed to emit settings event: {}", e)))?;
        
        Ok(())
    }
    
    /// Handle exit menu item click
    fn handle_exit_click(app: &AppHandle) -> AppResult<()> {
        log::info!("Exit menu clicked");
        
        // Emit event for cleanup before exit
        let _ = app.emit("app_exit_requested", ());
        
        // Exit the application
        app.exit(0);
        Ok(())
    }
    
    /// Handle profile menu item click
    fn handle_profile_click(app: &AppHandle, profile_name: &str) -> AppResult<()> {
        log::info!("Profile menu clicked: {}", profile_name);
        
        // Emit event with profile name
        app.emit("profile_switch_requested", profile_name)
            .map_err(|e| AppError::TrayError(format!("Failed to emit profile switch event: {}", e)))?;
        
        Ok(())
    }
    
    /// Show temporary status in menu item (e.g., ❕ during switch)
    pub fn update_profile_status(&mut self, profile_name: &str, status: &str) -> AppResult<()> {
        log::debug!("Updating profile status: {} -> {}", profile_name, status);
        
        // For temporary status updates (like showing ❕ during switch)
        // we need to rebuild the menu with updated text
        if let Some(_current_menu) = &self.current_menu {
            // Emit event to trigger menu refresh with temporary status
            self.app_handle.emit("profile_status_update", (profile_name, status))
                .map_err(|e| AppError::TrayError(format!("Failed to emit status update: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Update menu with profiles and temporary status indicators
    pub fn update_menu_with_status(&mut self, profiles: &[Profile], status_updates: &std::collections::HashMap<String, String>) -> AppResult<()> {
        log::info!("Updating tray menu with {} profiles and status updates", profiles.len());
        
        let mut menu_builder = MenuBuilder::new(&self.app_handle);
        
        // Add profile menu items with status
        for profile in profiles {
            let menu_text = if let Some(temp_status) = status_updates.get(&profile.name) {
                // Show temporary status (e.g., "Profile ❕")
                format!("{} {}", profile.name, temp_status)
            } else if profile.is_active {
                // Show active status
                format!("{} ✅", profile.name)
            } else {
                // No status
                profile.name.clone()
            };
            
            let menu_item = MenuItemBuilder::with_id(
                format!("profile_{}", profile.name),
                menu_text
            ).build(&self.app_handle)?;
            
            menu_builder = menu_builder.item(&menu_item);
        }
        
        // Add separator and system menu items
        let menu = menu_builder
            .separator()
            .item(&MenuItemBuilder::with_id("settings", "Settings").build(&self.app_handle)?)
            .item(&MenuItemBuilder::with_id("exit", "Exit").build(&self.app_handle)?)
            .build()?;
        
        // Update the tray menu - get tray by ID
        if let Some(tray) = self.app_handle.tray_by_id(&self.tray_id) {
            tray.set_menu(Some(menu.clone()))?;
        }
        
        self.current_menu = Some(menu);
        log::info!("Tray menu updated successfully with status indicators");
        
        Ok(())
    }
    
    /// Force refresh the menu (useful after profile changes)
    pub fn refresh_menu(&mut self, profiles: &[Profile]) -> AppResult<()> {
        self.update_menu(profiles)
    }
    
    /// Set tray tooltip
    pub fn set_tooltip(&self, text: &str) -> AppResult<()> {
        if let Some(tray) = self.app_handle.tray_by_id(&self.tray_id) {
            tray.set_tooltip(Some(text))?;
        }
        Ok(())
    }
    
    /// Get current menu reference
    pub fn get_current_menu(&self) -> Option<&Menu<tauri::Wry>> {
        self.current_menu.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Profile;
    use std::path::PathBuf;
    
    fn create_test_profile(name: &str, is_active: bool) -> Profile {
        Profile {
            name: name.to_string(),
            path: PathBuf::from(format!("{}.settings.json", name)),
            content: "{}".to_string(),
            is_active,
        }
    }
    
    // Note: Testing TrayService requires a Tauri app context
    // These tests would need to be integration tests with a real Tauri app
    // For now, we'll test the basic logic
    
    #[test]
    fn test_profile_creation() {
        let profile = create_test_profile("test", true);
        assert_eq!(profile.name, "test");
        assert!(profile.is_active);
    }
    
    #[test]
    fn test_menu_text_generation() {
        let active_profile = create_test_profile("active", true);
        let inactive_profile = create_test_profile("inactive", false);
        
        // Test that active profiles would have checkmark
        let active_text = if active_profile.is_active {
            format!("{} ✅", active_profile.name)
        } else {
            active_profile.name.clone()
        };
        
        let inactive_text = if inactive_profile.is_active {
            format!("{} ✅", inactive_profile.name)
        } else {
            inactive_profile.name.clone()
        };
        
        assert_eq!(active_text, "active ✅");
        assert_eq!(inactive_text, "inactive");
    }
}