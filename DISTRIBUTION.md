# BestMe Distribution Guide

This guide covers how to build and distribute BestMe across different platforms.

## Prerequisites

### All Platforms
- Node.js 20+ and npm
- Rust 1.75+ (via rustup)
- Git

### Windows
- Visual Studio 2022 with C++ tools
- WebView2 (usually pre-installed on Windows 10/11)

### macOS
- Xcode Command Line Tools
- Apple Developer Certificate (for signing)

### Linux
- Development libraries:
  ```bash
  sudo apt-get install libgtk-3-dev libwebkit2gtk-4.0-dev \
    libayatana-appindicator3-dev librsvg2-dev libasound2-dev \
    libclang-dev cmake
  ```

## Building for Production

### Quick Build (Current Platform)
```bash
./scripts/build_production.sh
```

### Platform-Specific Builds

#### Windows
```bash
./scripts/build_windows.sh
# Output: src-tauri/target/release/bundle/msi/*.msi
#         src-tauri/target/release/bundle/nsis/*.exe
```

#### macOS
```bash
./scripts/build_macos.sh
# Output: src-tauri/target/release/bundle/dmg/*.dmg
#         src-tauri/target/release/bundle/macos/BestMe.app
```

#### Linux
```bash
./scripts/build_linux.sh
# Output: src-tauri/target/release/bundle/appimage/*.AppImage
#         src-tauri/target/release/bundle/deb/*.deb
```

## Code Signing

### Windows
1. Obtain a code signing certificate
2. Set the certificate thumbprint in tauri.conf.json:
   ```json
   "windows": {
     "certificateThumbprint": "YOUR_CERT_THUMBPRINT"
   }
   ```

### macOS
1. Enroll in Apple Developer Program
2. Create a Developer ID certificate
3. Tauri will automatically use available certificates

## Distribution Channels

### 1. GitHub Releases (Recommended)
- Tag a release: `git tag -a v1.0.0 -m "Release v1.0.0"`
- Push tag: `git push origin v1.0.0`
- GitHub Actions will build and create release automatically

### 2. Direct Download
Host the installers on your website:
- Windows: `.msi` or `.exe` installer
- macOS: `.dmg` disk image
- Linux: `.AppImage` (universal) or `.deb`/`.rpm`

### 3. Package Managers

#### Windows (Chocolatey)
Create a `.nuspec` file and submit to Chocolatey community repository.

#### macOS (Homebrew)
Create a formula and submit to homebrew-cask.

#### Linux
- **Snap Store**: `snapcraft push bestme_*.snap`
- **Flathub**: Submit manifest to flathub/flathub repository
- **AUR**: Create PKGBUILD for Arch Linux

## Auto-Updates

### Setup
1. Generate update keys:
   ```bash
   npm run tauri signer generate
   ```
2. Add public key to tauri.conf.json
3. Configure update endpoint (e.g., GitHub releases)

### Testing Updates
1. Build version 1.0.0
2. Install it
3. Build version 1.0.1
4. Upload to update endpoint
5. Verify auto-update works

## Telemetry and Analytics

Consider adding (with user consent):
- Crash reporting (e.g., Sentry)
- Usage analytics (privacy-focused)
- Update success/failure rates

## Distribution Checklist

Before each release:
- [ ] Update version numbers
- [ ] Run all tests
- [ ] Build for all platforms
- [ ] Test installers
- [ ] Sign binaries
- [ ] Create release notes
- [ ] Upload to distribution channels
- [ ] Update website/documentation
- [ ] Monitor for issues

## Troubleshooting

### Build Failures
- Clear cache: `cargo clean && rm -rf ui/node_modules`
- Update dependencies: `cargo update && npm update`
- Check system requirements

### Code Signing Issues
- Windows: Verify certificate is valid and not expired
- macOS: Check `security find-identity -p codesigning`

### Distribution Issues
- Verify file permissions (especially for Linux)
- Check antivirus false positives (Windows)
- Ensure all dependencies are bundled