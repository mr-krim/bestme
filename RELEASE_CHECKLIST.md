# Release Checklist for BestMe

This checklist ensures consistent and high-quality releases for BestMe.

## Pre-Release Checklist

### 1. Code Quality
- [ ] All tests pass: `cargo test && cd ui && npm test`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Code is formatted: `cargo fmt && cd ui && npm run format`
- [ ] Dependencies are up to date: `cargo update && cd ui && npm update`
- [ ] No security vulnerabilities: `cargo audit && cd ui && npm audit`

### 2. Documentation
- [ ] CHANGELOG.md is updated with all changes
- [ ] README.md is current
- [ ] All new features are documented
- [ ] API changes are documented

### 3. Version Bumping
- [ ] Update version in `Cargo.toml`
- [ ] Update version in `src-tauri/Cargo.toml`
- [ ] Update version in `ui/package.json`
- [ ] Update version in `src-tauri/tauri.conf.json`
- [ ] Commit version changes: `git commit -am "chore: bump version to X.Y.Z"`

### 4. Testing
- [ ] Manual testing on Windows
- [ ] Manual testing on macOS
- [ ] Manual testing on Linux
- [ ] AI features work correctly
- [ ] Voice commands work correctly
- [ ] System tray functionality works
- [ ] Settings persistence works

## Build Process

### 1. Clean Build
```bash
# Clean previous builds
cargo clean
cd ui && rm -rf node_modules dist && npm install

# Run production build
./scripts/build_production.sh
```

### 2. Platform-Specific Builds
- [ ] Windows: `./scripts/build_windows.sh`
- [ ] macOS: `./scripts/build_macos.sh`
- [ ] Linux: `./scripts/build_linux.sh`

### 3. Test Built Binaries
- [ ] Windows installer works correctly
- [ ] macOS DMG mounts and app runs
- [ ] Linux AppImage runs on multiple distros
- [ ] Auto-update works (if implemented)

## Release Process

### 1. Create Git Tag
```bash
git tag -a v1.0.0 -m "Release version 1.0.0"
git push origin v1.0.0
```

### 2. GitHub Release
- [ ] GitHub Actions workflow completes successfully
- [ ] All platform binaries are uploaded
- [ ] Release notes are comprehensive
- [ ] Mark pre-release if appropriate

### 3. Distribution
- [ ] Update website with new version
- [ ] Update documentation site
- [ ] Announce on social media
- [ ] Notify beta testers

## Post-Release

### 1. Monitor
- [ ] Check for crash reports
- [ ] Monitor GitHub issues
- [ ] Check telemetry (if implemented)

### 2. Hotfix Process
If critical issues are found:
1. Create hotfix branch from release tag
2. Fix the issue
3. Test thoroughly
4. Create new patch version (e.g., v1.0.1)
5. Follow expedited release process

## Version Numbering

We follow Semantic Versioning (SemVer):
- MAJOR version: Incompatible API changes
- MINOR version: New functionality, backwards compatible
- PATCH version: Bug fixes, backwards compatible

Example: v1.2.3
- 1 = Major version
- 2 = Minor version  
- 3 = Patch version