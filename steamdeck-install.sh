#!/bin/bash

# DeckFlix Steam Deck Installation Script
# This script automates the entire installation process

set -e  # Exit on error

echo "======================================"
echo "  DeckFlix Steam Deck Installer"
echo "======================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored messages
print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}→ $1${NC}"
}

# Check if running on Steam Deck
print_info "Checking if running on Steam Deck..."
if [ ! -f /etc/os-release ] || ! grep -q "steamdeck" /etc/os-release; then
    print_error "This script is designed for Steam Deck only!"
    exit 1
fi
print_success "Steam Deck detected"

# Disable read-only filesystem
print_info "Disabling read-only filesystem..."
sudo steamos-readonly disable
print_success "Read-only disabled"

# Initialize pacman keys
print_info "Initializing pacman keys (this may take a while)..."
sudo rm -rf /etc/pacman.d/gnupg 2>/dev/null || true
sudo pacman-key --init
sudo pacman-key --populate archlinux
sudo pacman-key --populate holo
print_success "Pacman keys initialized"

# Update keyring
print_info "Updating keyring..."
sudo pacman -Sy archlinux-keyring holo-keyring --noconfirm
print_success "Keyring updated"

# Install all required system dependencies
print_info "Installing system dependencies..."
sudo pacman -S --needed base-devel gtk3 glib2 gobject-introspection cairo pango atk gdk-pixbuf2 webkit2gtk libsoup libappindicator-gtk3 librsvg pkg-config openssl nodejs npm --noconfirm
print_success "System dependencies installed"

# Install Rust
print_info "Installing Rust..."
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
    print_success "Rust installed"
else
    print_success "Rust already installed"
fi

# Configure npm for user installation
print_info "Configuring npm..."
mkdir -p ~/.npm-global
npm config set prefix '~/.npm-global'
if ! grep -q "npm-global/bin" ~/.bashrc; then
    echo 'export PATH=~/.npm-global/bin:$PATH' >> ~/.bashrc
fi
export PATH=~/.npm-global/bin:$PATH
print_success "npm configured"

# Install Peerflix
print_info "Installing Peerflix..."
npm install -g peerflix
print_success "Peerflix installed"

# Install MPV
print_info "Installing MPV video player..."
if ! command -v mpv &> /dev/null; then
    sudo pacman -S mpv --noconfirm
    print_success "MPV installed"
else
    print_success "MPV already installed"
fi

# Re-enable read-only filesystem
print_info "Re-enabling read-only filesystem..."
sudo steamos-readonly enable
print_success "Read-only re-enabled"

# Navigate to deckflix directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/deckflix"

print_info "Installing Node.js dependencies..."
npm install
print_success "Dependencies installed"

# Set PKG_CONFIG_PATH
export PKG_CONFIG_PATH=/usr/lib/pkgconfig:/usr/share/pkgconfig

# Build the application
print_info "Building DeckFlix (this will take 10-30 minutes)..."
print_info "Please be patient, this is normal for the first build..."
npm run tauri build

if [ $? -eq 0 ]; then
    print_success "Build completed successfully!"
    echo ""
    echo "======================================"
    echo "  Installation Complete!"
    echo "======================================"
    echo ""
    echo "To run DeckFlix:"
    echo "  cd $SCRIPT_DIR/deckflix"
    echo "  ./src-tauri/target/release/deckflix"
    echo ""
    echo "Or run: npm run tauri dev"
    echo ""
    echo "To add to Steam:"
    echo "  1. Open Steam in Desktop Mode"
    echo "  2. Games → Add Non-Steam Game"
    echo "  3. Browse to: $SCRIPT_DIR/deckflix/src-tauri/target/release/deckflix"
    echo ""
else
    print_error "Build failed! Check the errors above."
    exit 1
fi
