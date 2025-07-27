// Internationalization service for CCCS
use crate::{AppError, AppResult};
use std::collections::HashMap;

pub struct I18nService {
    current_locale: String,
    text_resources: HashMap<String, HashMap<String, String>>,
}

impl I18nService {
    pub fn new() -> Self {
        let current_locale = Self::detect_system_locale();
        let mut service = Self {
            current_locale: current_locale.clone(),
            text_resources: HashMap::new(),
        };
        
        if let Err(e) = service.load_text_resources() {
            log::warn!("Failed to load text resources: {}", e);
        }
        
        service
    }
    
    /// Detect system locale
    pub fn detect_system_locale() -> String {
        log::info!("Detecting system locale...");
        
        // Try to get system locale from environment variables
        if let Ok(locale) = std::env::var("LANG") {
            log::info!("LANG env var: {}", locale);
            if locale.starts_with("zh") {
                log::info!("Detected Chinese locale from LANG");
                return "zh".to_string();
            }
        }
        
        // Try LC_ALL
        if let Ok(locale) = std::env::var("LC_ALL") {
            log::info!("LC_ALL env var: {}", locale);
            if locale.starts_with("zh") {
                log::info!("Detected Chinese locale from LC_ALL");
                return "zh".to_string();
            }
        }
        
        // Platform-specific detection
        #[cfg(target_os = "macos")]
        {
            if let Some(locale) = Self::get_macos_locale() {
                log::info!("macOS locale: {}", locale);
                if locale.starts_with("zh") {
                    log::info!("Detected Chinese locale from macOS");
                    return "zh".to_string();
                }
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            if let Some(locale) = Self::get_windows_locale() {
                log::info!("Windows locale: {}", locale);
                if locale.starts_with("zh") {
                    log::info!("Detected Chinese locale from Windows");
                    return "zh".to_string();
                }
            }
        }
        
        // Default to English
        log::info!("No Chinese locale detected, defaulting to English");
        "en".to_string()
    }
    
    #[cfg(target_os = "macos")]
    fn get_macos_locale() -> Option<String> {
        use std::process::Command;
        
        if let Ok(output) = Command::new("defaults")
            .args(&["read", "-g", "AppleLocale"])
            .output()
        {
            if let Ok(locale) = String::from_utf8(output.stdout) {
                return Some(locale.trim().to_string());
            }
        }
        None
    }
    
    #[cfg(target_os = "windows")]
    fn get_windows_locale() -> Option<String> {
        // This would require Windows API calls
        // For now, return None and fall back to environment variables
        None
    }
    
    /// Load text resources for all supported languages
    pub fn load_text_resources(&mut self) -> AppResult<()> {
        self.text_resources.clear();
        
        // English resources
        let mut en_resources = HashMap::new();
        en_resources.insert("app_name".to_string(), "CCCS".to_string());
        en_resources.insert("app_description".to_string(), "Claude Code Configuration Switcher".to_string());
        en_resources.insert("settings".to_string(), "Settings".to_string());
        en_resources.insert("exit".to_string(), "Exit".to_string());
        en_resources.insert("profile".to_string(), "Profile".to_string());
        en_resources.insert("active".to_string(), "Active".to_string());
        en_resources.insert("inactive".to_string(), "Inactive".to_string());
        en_resources.insert("switch_profile".to_string(), "Switch to profile: {}".to_string());
        en_resources.insert("switching_profile".to_string(), "Switching profile...".to_string());
        en_resources.insert("profile_switched".to_string(), "Profile switched successfully".to_string());
        en_resources.insert("switch_failed".to_string(), "Failed to switch profile".to_string());
        en_resources.insert("claude_not_found".to_string(), "Claude Code installation not found".to_string());
        en_resources.insert("settings_not_found".to_string(), "settings.json not found. Please run Claude Code at least once.".to_string());
        en_resources.insert("monitor_interval".to_string(), "Monitor interval: {} minutes".to_string());
        en_resources.insert("monitoring_started".to_string(), "File monitoring started".to_string());
        en_resources.insert("monitoring_stopped".to_string(), "File monitoring stopped".to_string());
        en_resources.insert("config_changed".to_string(), "Configuration file changed".to_string());
        en_resources.insert("error".to_string(), "Error".to_string());
        en_resources.insert("warning".to_string(), "Warning".to_string());
        en_resources.insert("info".to_string(), "Information".to_string());
        en_resources.insert("ok".to_string(), "OK".to_string());
        en_resources.insert("cancel".to_string(), "Cancel".to_string());
        en_resources.insert("close".to_string(), "Close".to_string());
        en_resources.insert("save".to_string(), "Save".to_string());
        en_resources.insert("loading".to_string(), "Loading...".to_string());
        en_resources.insert("saving".to_string(), "Saving...".to_string());
        
        // Chinese resources
        let mut zh_resources = HashMap::new();
        zh_resources.insert("app_name".to_string(), "CCCS".to_string());
        zh_resources.insert("app_description".to_string(), "Claude Code 配置切换器".to_string());
        zh_resources.insert("settings".to_string(), "设置".to_string());
        zh_resources.insert("exit".to_string(), "退出".to_string());
        zh_resources.insert("profile".to_string(), "配置".to_string());
        zh_resources.insert("active".to_string(), "激活".to_string());
        zh_resources.insert("inactive".to_string(), "未激活".to_string());
        zh_resources.insert("switch_profile".to_string(), "切换到配置: {}".to_string());
        zh_resources.insert("switching_profile".to_string(), "正在切换配置...".to_string());
        zh_resources.insert("profile_switched".to_string(), "配置切换成功".to_string());
        zh_resources.insert("switch_failed".to_string(), "配置切换失败".to_string());
        zh_resources.insert("claude_not_found".to_string(), "未找到 Claude Code 安装".to_string());
        zh_resources.insert("settings_not_found".to_string(), "未找到 settings.json 文件。请至少运行一次 Claude Code。".to_string());
        zh_resources.insert("monitor_interval".to_string(), "监控间隔: {} 分钟".to_string());
        zh_resources.insert("monitoring_started".to_string(), "文件监控已启动".to_string());
        zh_resources.insert("monitoring_stopped".to_string(), "文件监控已停止".to_string());
        zh_resources.insert("config_changed".to_string(), "配置文件已更改".to_string());
        zh_resources.insert("error".to_string(), "错误".to_string());
        zh_resources.insert("warning".to_string(), "警告".to_string());
        zh_resources.insert("info".to_string(), "信息".to_string());
        zh_resources.insert("ok".to_string(), "确定".to_string());
        zh_resources.insert("cancel".to_string(), "取消".to_string());
        zh_resources.insert("close".to_string(), "关闭".to_string());
        zh_resources.insert("save".to_string(), "保存".to_string());
        zh_resources.insert("loading".to_string(), "加载中...".to_string());
        zh_resources.insert("saving".to_string(), "保存中...".to_string());
        
        self.text_resources.insert("en".to_string(), en_resources);
        self.text_resources.insert("zh".to_string(), zh_resources);
        
        log::info!("Loaded text resources for {} languages", self.text_resources.len());
        Ok(())
    }
    
    /// Get text for a specific key
    pub fn get_text(&self, key: &str) -> String {
        self.get_text_with_args(key, &[])
    }
    
    /// Get text for a specific key with arguments for formatting
    pub fn get_text_with_args(&self, key: &str, args: &[&str]) -> String {
        let resources = self.text_resources.get(&self.current_locale)
            .or_else(|| self.text_resources.get("en"))
            .expect("English resources should always be available");
        
        if let Some(template) = resources.get(key) {
            // Simple string formatting - replace {} with arguments in order
            let mut result = template.clone();
            for arg in args {
                if let Some(pos) = result.find("{}") {
                    result.replace_range(pos..pos+2, arg);
                } else {
                    break;
                }
            }
            result
        } else {
            log::warn!("Missing translation for key '{}' in locale '{}'", key, self.current_locale);
            key.to_string()
        }
    }
    
    /// Get all supported locales
    pub fn get_supported_locales() -> Vec<String> {
        vec!["en".to_string(), "zh".to_string()]
    }
    
    /// Set the current locale
    pub fn set_locale(&mut self, locale: &str) -> AppResult<()> {
        if !Self::get_supported_locales().contains(&locale.to_string()) {
            return Err(AppError::I18nError(
                format!("Unsupported locale: {}", locale)
            ));
        }
        
        self.current_locale = locale.to_string();
        log::info!("Locale changed to: {}", locale);
        Ok(())
    }
    
    /// Get current locale
    pub fn get_current_locale(&self) -> &str {
        &self.current_locale
    }
    
    /// Check if a locale is supported
    pub fn is_locale_supported(locale: &str) -> bool {
        Self::get_supported_locales().contains(&locale.to_string())
    }
    
    /// Get localized profile menu text
    pub fn get_profile_menu_text(&self, profile_name: &str, is_active: bool) -> String {
        if is_active {
            format!("{} ✅", profile_name)
        } else {
            profile_name.to_string()
        }
    }
    
    /// Get localized tray tooltip
    pub fn get_tray_tooltip(&self, profile_count: usize, active_profile: Option<&str>) -> String {
        let base_tooltip = if profile_count == 0 {
            self.get_text("app_description")
        } else if let Some(active) = active_profile {
            format!("{} - {}: {}", 
                self.get_text("app_description"),
                self.get_text("active"),
                active
            )
        } else {
            format!("{} - {} {}", 
                self.get_text("app_description"),
                profile_count,
                if profile_count == 1 { 
                    self.get_text("profile") 
                } else { 
                    format!("{}s", self.get_text("profile"))
                }
            )
        };
        
        base_tooltip
    }
}

// Tauri commands for i18n
#[tauri::command]
pub async fn get_current_locale(state: tauri::State<'_, std::sync::Mutex<I18nService>>) -> Result<String, String> {
    let service = state.lock().map_err(|e| format!("Failed to lock i18n service: {}", e))?;
    Ok(service.get_current_locale().to_string())
}

#[tauri::command]
pub async fn set_locale(
    locale: String,
    state: tauri::State<'_, std::sync::Mutex<I18nService>>,
) -> Result<(), String> {
    let mut service = state.lock().map_err(|e| format!("Failed to lock i18n service: {}", e))?;
    service.set_locale(&locale).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_text(
    key: String,
    args: Option<Vec<String>>,
    state: tauri::State<'_, std::sync::Mutex<I18nService>>,
) -> Result<String, String> {
    let service = state.lock().map_err(|e| format!("Failed to lock i18n service: {}", e))?;
    
    if let Some(args) = args {
        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        Ok(service.get_text_with_args(&key, &args_refs))
    } else {
        Ok(service.get_text(&key))
    }
}

#[tauri::command]
pub async fn get_supported_locales() -> Result<Vec<String>, String> {
    Ok(I18nService::get_supported_locales())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_i18n_service_creation() {
        let service = I18nService::new();
        assert!(!service.current_locale.is_empty());
        assert!(!service.text_resources.is_empty());
    }
    
    #[test]
    fn test_get_text() {
        let service = I18nService::new();
        
        // Test basic text retrieval
        let text = service.get_text("app_name");
        assert_eq!(text, "CCCS");
        
        // Test missing key
        let missing = service.get_text("non_existent_key");
        assert_eq!(missing, "non_existent_key");
    }
    
    #[test]
    fn test_get_text_with_args() {
        let service = I18nService::new();
        
        // Test text with formatting
        let text = service.get_text_with_args("switch_profile", &["test"]);
        assert!(text.contains("test"));
    }
    
    #[test]
    fn test_set_locale() {
        let mut service = I18nService::new();
        
        // Test valid locale
        assert!(service.set_locale("zh").is_ok());
        assert_eq!(service.get_current_locale(), "zh");
        
        // Test invalid locale
        assert!(service.set_locale("invalid").is_err());
    }
    
    #[test]
    fn test_locale_specific_text() {
        let mut service = I18nService::new();
        
        // Test English
        service.set_locale("en").unwrap();
        assert_eq!(service.get_text("settings"), "Settings");
        
        // Test Chinese
        service.set_locale("zh").unwrap();
        assert_eq!(service.get_text("settings"), "设置");
    }
    
    #[test]
    fn test_get_supported_locales() {
        let locales = I18nService::get_supported_locales();
        assert!(locales.contains(&"en".to_string()));
        assert!(locales.contains(&"zh".to_string()));
    }
    
    #[test]
    fn test_profile_menu_text() {
        let service = I18nService::new();
        
        let active_text = service.get_profile_menu_text("test", true);
        assert_eq!(active_text, "test ✅");
        
        let inactive_text = service.get_profile_menu_text("test", false);
        assert_eq!(inactive_text, "test");
    }
    
    #[test]
    fn test_tray_tooltip() {
        let mut service = I18nService::new();
        service.set_locale("en").unwrap();
        
        // Test with no profiles
        let tooltip = service.get_tray_tooltip(0, None);
        assert!(tooltip.contains("Claude Code Configuration Switcher"));
        
        // Test with active profile
        let tooltip = service.get_tray_tooltip(2, Some("work"));
        assert!(tooltip.contains("Active"));
        assert!(tooltip.contains("work"));
    }
}