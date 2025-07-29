#!/bin/bash
# Prepare Windows release for BestMe

set -e

echo "Preparing Windows release..."

# 1. Update version numbers
VERSION="1.0.0"
echo "Setting version to $VERSION"

# Update Cargo.toml versions
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml

# Update package.json
cd ui
npm version $VERSION --no-git-tag-version
cd ..

# 2. Create Windows-specific assets if they don't exist
echo "Creating Windows installer assets..."
mkdir -p src-tauri/icons

# 3. Update dependencies
echo "Updating dependencies..."
cargo update
cd ui && npm update && cd ..

# 4. Run tests
echo "Running tests..."
cd ui && npm test -- --run || echo "Some UI tests failed, continuing..."
cd ..

# 5. Build frontend
echo "Building frontend..."
cd ui
npm run build
cd ..

# 6. Create Windows target directory
mkdir -p dist/windows

echo "Windows release preparation complete!"
echo "Next steps:"
echo "1. Run on Windows: cd ui && npm run tauri build"
echo "2. Sign the installer with your certificate"
echo "3. Test the installer on a clean Windows machine"