#!/usr/bin/env bash

# Exit immediately if a command exits with a non-zero status
set -e

echo "=== Building middle-button-doubleclick ==="
cargo build --release

echo ""
echo "=== Installing system files (requires sudo) ==="

# FIX: Stop the running service first so the file isn't "busy"
if systemctl is-active --quiet middle-button-doubleclick.service; then
    echo "Stopping existing service to update binary..."
    sudo systemctl stop middle-button-doubleclick.service
fi

# 1. Copy the compiled release binary to a global location
sudo cp target/release/middle-button-doubleclick /usr/local/bin/

# 2. Copy the systemd service from your repository folder to the system directory
sudo cp "Background Service/middle-button-doubleclick.service" /etc/systemd/system/

# 3. Force systemd to scan for the new file, enable it for boot, and start it right now
echo "=== Activating background service ==="
sudo systemctl daemon-reload
sudo systemctl enable middle-button-doubleclick.service
sudo systemctl restart middle-button-doubleclick.service

echo ""
echo "Successfully installed and started middle-button-doubleclick!"
echo "You can check its logs anytime using: journalctl -u middle-button-doubleclick.service -f"
