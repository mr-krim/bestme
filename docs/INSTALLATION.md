# BestMe Installation Guide

This guide will help you set up the development environment for BestMe, a modern cross-platform speech-to-text application built with Rust, Tauri, and Svelte.

## Prerequisites

- **Rust**: 1.70.0 or later with Cargo
- **Node.js**: 18.x or later with npm
- **Tauri CLI**: For building and running the application
- **Platform-specific dependencies**: See below for your operating system

## Quick Start

1. **Install Rust**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install Node.js**
   - Download from [nodejs.org](https://nodejs.org/) or use a version manager like nvm

3. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/bestme.git
   cd bestme
   ```

4. **Install dependencies**
   ```bash
   # Install Tauri CLI
   cargo install tauri-cli

   # Install frontend dependencies
   cd ui
   npm install
   cd ..
   ```

5. **Run the application**
   ```bash
   cargo tauri dev
   ```

## Platform-Specific Requirements

### Windows

#### Requirements
- Windows 10 version 1803 or later (Windows 11 recommended)
- Microsoft Visual C++ 2019 Redistributable
- WebView2 Runtime (automatically installed by Tauri)

#### Setup
1. Install Visual Studio Build Tools or Visual Studio Community
2. During installation, select "Desktop development with C++"
3. WebView2 will be automatically installed when running the app

### macOS

#### Requirements
- macOS 10.15 (Catalina) or later
- Xcode Command Line Tools

#### Setup
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

### Linux

#### Debian/Ubuntu
```bash
sudo apt-get update && sudo apt-get install -y \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libjavascriptcoregtk-4.1-dev \
    libsoup-3.0-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

#### Fedora
```bash
sudo dnf install webkit2gtk4.1-devel \
    gtk3-devel \
    libsoup3-devel \
    libappindicator-gtk3-devel \
    librsvg2-devel \
    javascriptcoregtk4.1-devel \
    openssl-devel
```

#### Arch Linux
```bash
sudo pacman -S webkit2gtk-4.1 \
    gtk3 \
    libsoup3 \
    libappindicator-gtk3 \
    librsvg \
    javascriptcoregtk-4.1 \
    base-devel \
    openssl
```

#### OpenSUSE
```bash
sudo zypper install webkit2gtk3-devel \
    gtk3-devel \
    libsoup3-devel \
    libappindicator3-devel \
    librsvg-devel \
    javascriptcoregtk4.1-devel \
    libopenssl-devel
```

### WSL (Windows Subsystem for Linux)

If using WSL for development:
1. Install the Linux dependencies for your distribution (see above)
2. For GUI support, ensure WSLg is enabled (WSL 2) or install an X server like VcXsrv
3. Set the DISPLAY environment variable if using an external X server

## Building for Production

### Development Build
```bash
cargo tauri dev
```

### Production Build
```bash
cargo tauri build
```

Built applications will be in `src-tauri/target/release/bundle/`:
- **Windows**: `.msi` installer
- **macOS**: `.dmg` disk image
- **Linux**: `.deb`, `.rpm`, and `.AppImage` packages

## Development Scripts

BestMe includes several helper scripts in the `scripts/` directory:

- `run_default.sh/.bat` - Run with default settings
- `run_voice.sh/.bat` - Run with voice commands enabled
- `run_debug.sh/.bat` - Run in debug mode
- `run_dev.sh/.bat` - Run in development mode

## Troubleshooting

### Build Errors
- Ensure all platform-specific dependencies are installed
- Update Rust: `rustup update`
- Clear build cache: `cargo clean`
- Reinstall node modules: `rm -rf node_modules && npm install`

### Runtime Errors
- **Windows**: Check Event Viewer for application logs
- **macOS**: Check Console.app for crash reports
- **Linux**: Run from terminal to see error output

### Common Issues

1. **"Cannot find module '@tauri-apps/api'"**
   - Run `npm install` in the `ui/` directory
   - Ensure you're using Tauri 2.0 compatible API

2. **WebView2 not found (Windows)**
   - Download from [Microsoft Edge WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

3. **Audio device errors**
   - Ensure microphone permissions are granted
   - Check audio device is properly connected

## Verifying Installation

Run these commands to verify your setup:

```bash
# Check Rust
rustc --version
cargo --version

# Check Node.js
node --version
npm --version

# Check Tauri CLI
cargo tauri --version

# Linux only: Check for required libraries
pkg-config --list-all | grep -E 'webkit2gtk-4.1|libsoup-3.0'
```

## Next Steps

1. Review the [DEVELOPMENT.md](DEVELOPMENT.md) for development guidelines
2. Check [VOICE_COMMANDS.md](VOICE_COMMANDS.md) for voice command usage
3. See [organization.md](organization.md) for project structure

## Support

If you encounter issues not covered here:
1. Check the [Tauri Documentation](https://tauri.app/)
2. Review our [GitHub Issues](https://github.com/yourusername/bestme/issues)
3. Join our community discussions