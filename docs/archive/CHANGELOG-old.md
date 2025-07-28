# Changelog

## [Unreleased]

### Fixed
- Windows GUI mode support after migration to Tauri 2.0
- Fixed code warnings and unreachable code in device.rs
- Improved audio device detection for Windows systems
- Updated main.rs to handle command-line arguments for GUI mode

### Added
- Added run-gui-mode.bat and run-gui-mode.ps1 scripts for easier launching on Windows
- Updated README-WINDOWS.md with improved instructions for Windows users
- Windows-specific audio device handling optimizations

### Changed
- Code cleanup to remove unused imports and reduce warnings
- Improved Windows-specific error handling

## [0.1.0] - 2024-04-10

### Added
- Initial release with basic functionality
- Audio capture and transcription
- GUI mode for Windows 
