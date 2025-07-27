// CCCS Settings Page JavaScript

// Tauri APIs should be available when this script loads
const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;

// Global state
let currentSettings = null;
let isLoading = false;

// DOM elements
const elements = {
    languageSelect: null,
    claudeDirectory: null,
    profilesCount: null,
    closeButton: null,
    loadingOverlay: null,
    toastContainer: null,
};

// Internationalization
const translations = {
    en: {
        settings_title: 'CCCS - Settings',
        app_info_title: 'Application Information',
        app_description: 'CCCS (Claude Code Configuration Switcher) is a tool for quickly switching Claude Code configuration files.',
        status_icons_title: 'Profile Status Icons',
        full_match_description: 'Complete match - configuration fully matches current settings',
        partial_match_description: 'Partial match - identical except model field (auto-updated by Claude Code)',
        error_status_description: 'Error - failed to read or parse configuration file',
        no_match_description: 'No icon - configuration differs from current settings',
        language_settings_title: 'Language Settings',
        interface_language_label: 'Interface language:',
        follow_system: 'Follow system',
        chinese: '中文',
        english: 'English',
        current_status_title: 'Current Status',
        claude_directory_label: 'Claude Code directory:',
        profiles_found_label: 'Configuration files found:',
        close_button: 'Close',
        saving_settings: 'Saving settings...',
        active: 'Active',
        inactive: 'Inactive',
        settings_saved: 'Settings saved successfully',
        settings_error: 'Error saving settings',
    },
    zh: {
        settings_title: 'CCCS - 设置',
        app_info_title: '应用程序信息',
        app_description: 'CCCS (Claude Code Configuration Switcher) 是一个用于快速切换 Claude Code 配置文件的工具。',
        status_icons_title: '配置状态图标',
        full_match_description: '完全匹配 - 配置与当前设置完全一致',
        partial_match_description: '部分匹配 - 除model字段外完全一致（Claude Code自动更新）',
        error_status_description: '错误 - 读取或解析配置文件失败',
        no_match_description: '无图标 - 配置与当前设置不同',
        language_settings_title: '语言设置',
        interface_language_label: '界面语言:',
        follow_system: '跟随系统',
        chinese: '中文',
        english: 'English',
        current_status_title: '当前状态',
        claude_directory_label: 'Claude Code 目录:',
        profiles_found_label: '发现的配置文件:',
        close_button: '关闭',
        saving_settings: '正在保存设置...',
        active: '激活',
        inactive: '未激活',
        settings_saved: '设置保存成功',
        settings_error: '保存设置时出错',
    }
};

let currentLanguage = 'en';

// Utility functions
function detectSystemLanguage() {
    const lang = navigator.language || navigator.languages[0];
    if (lang.startsWith('zh')) {
        return 'zh';
    }
    return 'en';
}

function updateTexts() {
    const t = translations[currentLanguage];
    
    document.querySelectorAll('[data-i18n]').forEach(element => {
        const key = element.getAttribute('data-i18n');
        if (t[key]) {
            element.textContent = t[key];
        }
    });
    
    // Update page title
    document.title = t.settings_title;
}

function showLoading(show) {
    console.log('showLoading called with:', show);
    isLoading = show;
    if (elements.loadingOverlay) {
        elements.loadingOverlay.style.display = show ? 'flex' : 'none';
        console.log('Loading overlay display:', elements.loadingOverlay.style.display);
    } else {
        console.error('Loading overlay element not found!');
    }
}

function showToast(message, type = 'success') {
    const toast = document.createElement('div');
    toast.className = `toast ${type}`;
    toast.textContent = message;
    
    elements.toastContainer.appendChild(toast);
    
    // Trigger animation
    setTimeout(() => toast.classList.add('show'), 10);
    
    // Auto remove
    setTimeout(() => {
        toast.classList.remove('show');
        setTimeout(() => {
            if (toast.parentNode) {
                toast.parentNode.removeChild(toast);
            }
        }, 300);
    }, 3000);
}

// API functions
async function loadSettings() {
    try {
        currentSettings = await invoke('get_settings');
        updateUI();
    } catch (error) {
        console.error('Failed to load settings:', error);
        showToast(translations[currentLanguage].settings_error, 'error');
    }
}

async function loadAppStatus() {
    console.log('Loading app status...');
    try {
        // Get profiles count from backend
        const profilesData = await invoke('get_profiles_info');
        console.log('Profiles data received:', profilesData);
        
        elements.claudeDirectory.textContent = profilesData.claude_directory || '~/.claude';
        elements.profilesCount.textContent = profilesData.profiles_count.toString();
    } catch (error) {
        console.error('Failed to load app status:', error);
        // Use fallback values
        elements.claudeDirectory.textContent = '~/.claude';
        elements.profilesCount.textContent = '0';
    }
}


async function updateLanguage(language) {
    try {
        showLoading(true);
        await invoke('update_language', { language });
        if (currentSettings) {
            currentSettings.language = language;
        }
        
        // Update current language and UI
        currentLanguage = language || detectSystemLanguage();
        updateTexts();
        
        showToast(translations[currentLanguage].settings_saved);
    } catch (error) {
        console.error('Failed to update language:', error);
        showToast(translations[currentLanguage].settings_error, 'error');
    } finally {
        showLoading(false);
    }
}

// UI update functions
function updateUI() {
    if (!currentSettings) return;
    
    elements.languageSelect.value = currentSettings.language || '';
    
    // Update language only if explicitly set in settings
    if (currentSettings.language) {
        currentLanguage = currentSettings.language;
        updateTexts();
    }
}

// Event handlers
function setupEventListeners() {
    // Language select
    elements.languageSelect.addEventListener('change', () => {
        const value = elements.languageSelect.value || null;
        updateLanguage(value);
    });
    
    // Close button
    elements.closeButton.addEventListener('click', async () => {
        console.log('Close button clicked');
        try {
            const window = getCurrentWindow();
            console.log('Got window instance:', window);
            await window.close();
            console.log('Window closed successfully');
        } catch (error) {
            console.error('Failed to close window:', error);
            // Fallback: try to close via API
            try {
                console.log('Trying to close via API...');
                await invoke('close_settings_window');
                console.log('Window closed via API');
            } catch (e) {
                console.error('Failed to close via API:', e);
            }
        }
    });
    
    // Prevent form submission on Enter
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
            e.preventDefault();
        }
    });
}

function initElements() {
    elements.languageSelect = document.getElementById('language-select');
    elements.claudeDirectory = document.getElementById('claude-directory');
    elements.profilesCount = document.getElementById('profiles-count');
    elements.closeButton = document.getElementById('close-button');
    elements.loadingOverlay = document.getElementById('loading-overlay');
    elements.toastContainer = document.getElementById('toast-container');
}

// Initialization
async function init() {
    try {
        initElements();
        
        // Get system language from backend first
        try {
            const locale = await invoke('get_current_locale');
            currentLanguage = locale || 'en';
            console.log('Backend detected locale:', currentLanguage);
        } catch (error) {
            console.error('Failed to get locale from backend:', error);
            currentLanguage = detectSystemLanguage();
        }
        
        updateTexts();
        setupEventListeners();
        
        // Load settings after language is set
        await loadSettings();
        await loadAppStatus();
        
        console.log('Settings page initialized successfully');
    } catch (error) {
        console.error('Failed to initialize settings page:', error);
        showToast('Failed to initialize settings page', 'error');
    }
}

// Start the application
init();