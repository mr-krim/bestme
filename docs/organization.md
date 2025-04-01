# BestMe Project Organization

This document outlines the organization of the BestMe project files and directories.

## Directory Structure

The project is organized into the following main directories:

- `src/`: Contains the Rust code for the core application
- `src-tauri/`: Contains the Tauri-specific code for the application
- `ui/`: Contains the frontend code for the application
- `config/`: Contains configuration files for the application
- `scripts/`: Contains scripts for building, running, and testing the application
- `docs/`: Contains documentation for the application
- `assets/`: Contains assets used by the application

## Rust Application Structure

The Rust code is organized as follows:

- `src/audio/`: Audio processing and recording functionality
  - `device.rs`: Audio device detection and management
  - `transcribe.rs`: Speech-to-text transcription
  - `voice_commands.rs`: Voice command detection and execution
- `src/config.rs`: Configuration management
- `src/main.rs`: Main application entry point
- `src/app.rs`: Application management

## Configuration Management

The application looks for configuration files in the following locations:

1. `<app_directory>/config/config.json`: Application-specific configuration
2. `~/.config/bestme/config.json`: User-specific configuration
3. Default configuration built into the application

The configuration loading process is handled by the `ConfigManager` in `src/config.rs`.

## Frontend Structure

The frontend code is organized in the `ui/` directory:

- `package.json`: Frontend dependencies and build scripts
- `tsconfig.json`: TypeScript configuration
- `index.html`: Main entry point for the frontend application
- `src/`: Source code for the frontend

## Scripts

The `scripts/` directory contains various scripts for building, running, and testing the application:

- `package-windows-wsl.sh`: Script for packaging the application for Windows from WSL
- `verify-core.sh`: Script for verifying that the core functionality of the application works
- `run-gui-mode.bat` and `run-gui-mode.ps1`: Scripts for running the application in GUI mode on Windows

## Documentation

The `docs/` directory contains documentation for the application:

- `development.md`: Development roadmap and plan
- `debugging.md`: Debugging information and common issues
- `organization.md`: This file, describing the project organization

## Benefits of the New Organization

The reorganization of the project offers several benefits:

1. **Clear Separation of Concerns**: Frontend, backend, and configuration are clearly separated.
2. **Improved Discoverability**: Files are grouped by their function, making it easier to find what you're looking for.
3. **Better Configuration Management**: The application checks multiple locations for configuration files, allowing for both application-specific and user-specific settings.
4. **Easier Maintenance**: With a well-defined structure, it's easier to add new features and maintain existing ones.
5. **Better Documentation**: Each major component has its own README file explaining its purpose and structure. 
