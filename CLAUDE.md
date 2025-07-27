# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CCCS (Claude Code Configuration Switcher) is a desktop system tray application built with Tauri that allows users to quickly switch between different Claude Code configuration profiles. The application runs in the background and provides real-time monitoring and switching capabilities through a system tray interface.

## Architecture

### Core Application Structure
- **Main App**: `src-tauri/src/app.rs` - Central application coordinator managing all services
- **Service-Based Architecture**: Modular design with dedicated services for different concerns
- **Async/Threaded Design**: Background monitoring and file system operations
- **System Integration**: Native system tray with cross-platform support

### Backend Services (src-tauri/src/)
- **ConfigService** (`config_service.rs`) - Manages Claude Code profile detection, parsing, and switching with file caching
- **TrayService** (`tray_service.rs`) - System tray icon and menu management with profile status indicators
- **MonitorService** (`monitor_service.rs`) - Background file system monitoring for configuration changes
- **SettingsService** (`settings_service.rs`) - Application settings management and persistence
- **I18nService** (`i18n_service.rs`) - Internationalization support for English/Chinese
- **ClaudeDetector** (`claude_detector.rs`) - Automatic detection of Claude Code installation and config directory

### Frontend (src/)
- **Minimal UI**: Basic Vite/TypeScript frontend for settings window
- **System Tray Focus**: Primary interaction through system tray, not traditional UI
- **Settings Interface**: Accessible via tray menu for configuration

## Development Commands

### Full Application Development
- `npm run tauri:dev` - Start Tauri development with hot reload (recommended)
- `npm run tauri:build` - Build complete application for production

### Frontend Only (Limited Use)
- `npm run dev` - Vite dev server (settings window only)
- `npm run build` - Build frontend assets
- `npm run preview` - Preview built frontend

### Rust Backend Testing
- `cargo test` (in src-tauri directory) - Run Rust unit tests
- `cargo check` (in src-tauri directory) - Check compilation without building

## Key Configuration Files

- `src-tauri/tauri.conf.json` - Main Tauri configuration including tray settings and build commands
- `src-tauri/Cargo.toml` - Rust dependencies including Tauri plugins (fs, dialog, log)
- `package.json` - Frontend dependencies and unified npm scripts
- `tsconfig.json` - TypeScript configuration

## Application Behavior

### Profile Detection Logic
- Scans `~/.claude/` directory for `*.settings.json` files (excluding main `settings.json`)
- Compares profiles against current active configuration
- Ignores `model` field differences (auto-updated by Claude Code)
- Provides status indicators: ✅ Full Match, 🔄 Partial Match, ❌ Error

### System Integration
- **macOS**: Runs as accessory app (no dock icon) with system tray
- **Cross-platform**: Windows and Linux support through Tauri
- **Background Operation**: Minimal resource usage with configurable monitoring intervals

## Important Development Notes

- **Tray-First Design**: The application is primarily accessed through system tray, not traditional windows
- **Error Resilience**: Services continue operating even if individual components fail
- **Async Initialization**: App startup is non-blocking with deferred service initialization
- **File System Monitoring**: Real-time detection of configuration file changes
- **Multi-language**: Built-in support for English and Chinese interfaces