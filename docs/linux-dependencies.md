# Linux Dependencies for BestMe

To build the BestMe application on Linux, you'll need to install several system dependencies.

## Debian/Ubuntu

Run the following command to install all required dependencies:

```bash
sudo apt-get update && sudo apt-get install -y \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libjavascriptcoregtk-4.1-dev \
    libsoup-3.0-dev
```

## Fedora

```bash
sudo dnf install webkit2gtk4.1-devel \
    gtk3-devel \
    libsoup3-devel \
    libappindicator-gtk3-devel \
    librsvg2-devel \
    javascriptcoregtk4.1-devel
```

## Arch Linux

```bash
sudo pacman -S webkit2gtk-4.1 \
    gtk3 \
    libsoup3 \
    libappindicator-gtk3 \
    librsvg \
    javascriptcoregtk-4.1
```

## OpenSUSE

```bash
sudo zypper install webkit2gtk3-devel \
    gtk3-devel \
    libsoup3-devel \
    libappindicator3-devel \
    librsvg-devel \
    javascriptcoregtk4.1-devel
```

After installing these dependencies, you should be able to build the application successfully.

## Verifying Installation

You can verify if the required packages are correctly installed by checking for the pkg-config files:

```bash
pkg-config --list-all | grep -E 'javascriptcoregtk-4.1|libsoup-3.0'
```

If correctly installed, this should return information about these packages.

## Note on WSL

If you're using Windows Subsystem for Linux (WSL), you'll need to ensure you have a proper X server configured if you want to run GUI applications. Tools like VcXsrv, Xming, or WSLg (for WSL 2) can be used for this purpose. 
