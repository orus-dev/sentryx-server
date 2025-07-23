#!/bin/bash
set -e

REPO_URL="https://github.com/orus-dev/sentryx-server.git"
INSTALL_DIR="$HOME/sentryx-server"  # expanded
SERVICE_NAME="sentryx-server"
SERVICE_FILE="/etc/systemd/system/$SERVICE_NAME.service"
CARGO_BIN="$(which cargo)"
RUN_USER="$(whoami)"  # change if needed

# Backup if exists
if [ -d "$INSTALL_DIR" ]; then
    echo "Sentryx already exists, backing up..."
    rm -rf "$INSTALL_DIR-backup"
    mv "$INSTALL_DIR" "$INSTALL_DIR-backup"
fi

# Clone as current user
git clone "$REPO_URL" "$INSTALL_DIR"

# Build as current user inside INSTALL_DIR
$CARGO_BIN build --release --manifest-path "$INSTALL_DIR/Cargo.toml"

# Create systemd service file with absolute ExecStart path
echo "Creating systemd service..."
sudo tee "$SERVICE_FILE" > /dev/null <<EOF
[Unit]
Description=SentryX Server
After=network.target

[Service]
Type=simple
User=$RUN_USER
WorkingDirectory=$INSTALL_DIR
ExecStart=$INSTALL_DIR/target/release/sentryx-server
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd and enable service
sudo systemctl daemon-reload
sudo systemctl enable --now "$SERVICE_NAME.service"

echo "Done! Check service with: sudo systemctl status $SERVICE_NAME"
