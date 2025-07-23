#!/bin/bash
set -e

SERVICE_NAME="sentryx-server"
SERVICE_FILE="/etc/systemd/system/$SERVICE_NAME.service"
INSTALL_DIR="$HOME/sentryx-server"  # Make sure this matches your install script

echo "Stopping and disabling systemd service..."
sudo systemctl stop "$SERVICE_NAME.service" || true
sudo systemctl disable "$SERVICE_NAME.service" || true

echo "Removing systemd service file..."
sudo rm -f "$SERVICE_FILE"

echo "Reloading systemd daemon..."
sudo systemctl daemon-reload

echo "Removing installation directory..."
rm -rf "$INSTALL_DIR"
rm -rf "${INSTALL_DIR}-backup"

echo "Uninstall complete."
