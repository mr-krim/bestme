#!/bin/bash

# This script installs the necessary dependencies for building the BestMe application on Linux

# Function to check for errors
check_error() {
    if [ $? -ne 0 ]; then
        echo "Error: $1"
        exit 1
    fi
}

# Determine Linux distribution
if [ -f /etc/os-release ]; then
    . /etc/os-release
    distro=$ID
elif [ -f /etc/lsb-release ]; then
    . /etc/lsb-release
    distro=$DISTRIB_ID
else
    distro="unknown"
fi

echo "Detected Linux distribution: $distro"

# Install dependencies based on distribution
case $distro in
    ubuntu|debian)
        echo "Installing dependencies for Debian/Ubuntu..."
        sudo apt-get update
        check_error "Failed to update package lists"
        
        sudo apt-get install -y \
            libwebkit2gtk-4.1-dev \
            libgtk-3-dev \
            libayatana-appindicator3-dev \
            librsvg2-dev \
            libjavascriptcoregtk-4.1-dev \
            libsoup-3.0-dev
        check_error "Failed to install dependencies"
        ;;
        
    fedora)
        echo "Installing dependencies for Fedora..."
        sudo dnf install -y \
            webkit2gtk4.1-devel \
            gtk3-devel \
            libsoup3-devel \
            libappindicator-gtk3-devel \
            librsvg2-devel \
            javascriptcoregtk4.1-devel
        check_error "Failed to install dependencies"
        ;;
        
    arch|manjaro)
        echo "Installing dependencies for Arch Linux..."
        sudo pacman -S --noconfirm \
            webkit2gtk-4.1 \
            gtk3 \
            libsoup3 \
            libappindicator-gtk3 \
            librsvg \
            javascriptcoregtk-4.1
        check_error "Failed to install dependencies"
        ;;
        
    opensuse|suse)
        echo "Installing dependencies for OpenSUSE..."
        sudo zypper install -y \
            webkit2gtk3-devel \
            gtk3-devel \
            libsoup3-devel \
            libappindicator3-devel \
            librsvg-devel \
            javascriptcoregtk4.1-devel
        check_error "Failed to install dependencies"
        ;;
        
    *)
        echo "Unsupported distribution: $distro"
        echo "Please install the dependencies manually. See bestme/docs/linux-dependencies.md for details."
        exit 1
        ;;
esac

echo "Verifying installation..."
pkg-config --list-all | grep -E 'javascriptcoregtk-4.1|libsoup-3.0'
check_error "Package verification failed"

echo "All dependencies have been installed successfully!"
echo "You can now build the BestMe application."

exit 0 
