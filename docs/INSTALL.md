# BestMe Installation Guide

This guide will help you set up the development environment for BestMe, a modern cross-platform speech-to-text application built with Rust, Tauri, and Svelte.

## Prerequisites

- **Rust**: 1.70.0 or later with Cargo
- **Node.js**: 18.x or later with npm
- **Tauri CLI**: For building and running the application
- **Platform-specific dependencies**: For Windows, macOS, or Linux

## Step 1: Install Rust

Follow the instructions at https://www.rust-lang.org/tools/install to install Rust and Cargo.

## Step 2: Install Node.js and npm

Download and install Node.js and npm from https://nodejs.org/

## Step 3: Install Tauri CLI

```bash
cargo install tauri-cli
```

## Step 4: Clone the repository

```bash
git clone https://github.com/yourusername/bestme.git
cd bestme
```

## Step 5: Install frontend dependencies

```bash
cd ui
npm install
cd ..
```

## Step 6: Build and run the application

For development:

```bash
cargo tauri dev
```

For production:

```bash
cargo tauri build
```

## Platform-Specific Requirements

### Windows

- Windows 10 or later
- Microsoft Visual C++ 2019 Redistributable
- WebView2 Runtime (automatically installed by Tauri)

### macOS

- macOS 10.15 or later
- Xcode Command Line Tools

### Linux

- A recent distribution (Ubuntu 20.04, Fedora 36, etc.)
- The following packages:
  - Ubuntu/Debian: `libwebkit2gtk-4.0-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev`
  - Fedora: `webkit2gtk4.0-devel gtk3-devel libappindicator-gtk3-devel librsvg2-devel`
  - Arch: `webkit2gtk base-devel gtk3 libappindicator-gtk3 librsvg`

## Troubleshooting

- **Build errors**: Make sure you have the latest version of Rust and the required system dependencies.
- **Runtime errors**: Check the logs in the Console (macOS), Event Viewer (Windows), or standard output (Linux).

## Development Workflow

1. Make changes to the Rust code in the `bestme` directory
2. Make changes to the Tauri backend in the `bestme/src-tauri` directory
3. Make changes to the frontend in the `bestme/ui` directory
4. Run `cargo tauri dev` to see your changes in action

## Further Documentation

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Svelte Documentation](https://svelte.dev/docs)
- [Rust Documentation](https://doc.rust-lang.org/book/) 
