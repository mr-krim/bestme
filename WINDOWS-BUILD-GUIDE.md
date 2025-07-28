# Windows Build Guide for BestMe

## 🎯 Quick Start for Windows Users

Since cross-compilation from Linux requires additional tools, here's the easiest way to build BestMe on Windows:

### Prerequisites

1. **Install Rust** (if not already installed):
   ```powershell
   # Download from https://rustup.rs/
   # Or use PowerShell:
   winget install Rustlang.Rustup
   ```

2. **Install Node.js** (v18 or later):
   ```powershell
   winget install OpenJS.NodeJS
   ```

3. **Install Git**:
   ```powershell
   winget install Git.Git
   ```

4. **Install Visual Studio Build Tools** (for Windows compilation):
   ```powershell
   winget install Microsoft.VisualStudio.2022.BuildTools
   ```
   - During installation, select "Desktop development with C++"

### Building BestMe

1. **Clone the repository**:
   ```powershell
   git clone https://github.com/yourusername/bestme.git
   cd bestme
   ```

2. **Install dependencies**:
   ```powershell
   # Install frontend dependencies
   cd ui
   npm install
   cd ..
   
   # Add Windows target (if needed)
   rustup target add x86_64-pc-windows-msvc
   ```

3. **Build the application**:
   ```powershell
   # Development build (faster, includes console)
   cargo tauri dev
   
   # Production build (creates installer)
   cargo tauri build
   ```

### Build Outputs

After successful build, you'll find:

1. **MSI Installer**: 
   `src-tauri\target\release\bundle\msi\BestMe_0.1.0_x64_en-US.msi`

2. **NSIS Installer**: 
   `src-tauri\target\release\bundle\nsis\BestMe_0.1.0_x64-setup.exe`

3. **Portable EXE**: 
   `src-tauri\target\release\bestme-tauri.exe`

## 🚀 Quick Test Build

For testing without full packaging:

```powershell
# Build only the executable
cd src-tauri
cargo build --release

# Run it
..\target\release\bestme-tauri.exe
```

## 📦 Creating a Portable Version

1. Create a new folder: `BestMe-Portable`
2. Copy these files:
   - `target\release\bestme-tauri.exe` → `BestMe.exe`
   - `config\` folder
   - Any required DLLs from `target\release\`

3. Create `run.bat`:
   ```batch
   @echo off
   echo Starting BestMe...
   BestMe.exe
   ```

## 🐛 Troubleshooting

### "WebView2 Runtime not found"
- Download from: https://developer.microsoft.com/en-us/microsoft-edge/webview2/
- Most Windows 10/11 systems have it pre-installed

### "VCRUNTIME140.dll not found"
- Install Visual C++ Redistributables:
  ```powershell
  winget install Microsoft.VCRedist.2015+.x64
  ```

### Build fails with "linker not found"
- Ensure Visual Studio Build Tools are installed with C++ workload
- Restart your terminal after installation

## 🎨 Development Tips

1. **Fast iteration**: Use `cargo tauri dev` for hot-reload
2. **Console output**: Add `--features console` for debug builds
3. **Custom icons**: Replace files in `src-tauri/icons/`

## 📝 Build Script for CI/CD

Save as `build-windows.ps1`:

```powershell
# Windows build script
$ErrorActionPreference = "Stop"

Write-Host "Building BestMe for Windows..." -ForegroundColor Green

# Build frontend
Set-Location ui
npm install
npm run build
Set-Location ..

# Build Tauri app
Set-Location src-tauri
cargo tauri build

# Copy outputs
$dist = "..\dist\windows"
New-Item -ItemType Directory -Force -Path $dist

Copy-Item "target\release\bundle\msi\*.msi" $dist
Copy-Item "target\release\bundle\nsis\*.exe" $dist
Copy-Item "target\release\bestme-tauri.exe" "$dist\BestMe-Portable.exe"

Write-Host "Build complete! Check dist\windows\" -ForegroundColor Green
```

## 🔄 Alternative: Use GitHub Actions

The easiest way is to use GitHub Actions for Windows builds. See `.github/workflows/release.yml` for automated builds on all platforms.

---

**Note**: For the best experience, build on the target platform. Windows builds are best created on Windows machines.