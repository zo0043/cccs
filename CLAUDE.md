# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Tauri application with a TypeScript/Vite frontend. The project combines a Rust backend (Tauri) with a web frontend built using Vite and TypeScript.

## Architecture

### Frontend (src/)
- **Entry point**: `src/main.ts` - Sets up the basic HTML structure and initializes the counter component
- **Components**: `src/counter.ts` - Simple counter functionality with click handlers
- **Styling**: `src/style.css` - Application styles
- **Build tool**: Vite with TypeScript support

### Backend (src-tauri/)
- **Entry point**: `src-tauri/src/main.rs` - Launches the Tauri application
- **Core logic**: `src-tauri/src/lib.rs` - Contains the main Tauri application setup with logging configuration
- **Build configuration**: `src-tauri/Cargo.toml` - Rust dependencies and build settings
- **Tauri configuration**: `src-tauri/tauri.conf.json` - App window settings, build commands, and bundle configuration

## Development Commands

### Frontend Development
- `npm run dev` - Start development server (runs Vite dev server on localhost:5173)
- `npm run build` - Build frontend for production (TypeScript compilation + Vite build)
- `npm run preview` - Preview production build

### Tauri Development
- Development is handled through the Tauri configuration in `tauri.conf.json`
- Frontend dev server runs on `http://localhost:5173`
- Build process: `npm run build` creates the `ui` directory for Tauri

## Key Configuration Files

- `package.json` - Frontend dependencies and npm scripts
- `tsconfig.json` - TypeScript configuration with strict settings
- `src-tauri/tauri.conf.json` - Tauri app configuration including window settings and build commands
- `src-tauri/Cargo.toml` - Rust dependencies and library configuration

## Project Structure Notes

- Frontend assets are served from the `public/` directory
- Tauri icons are stored in `src-tauri/icons/`
- The app uses a hybrid architecture where the frontend is built with Vite and bundled into the Tauri application
- No test framework is currently configured