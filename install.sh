#!/bin/bash

set -e

REPO_URL="https://github.com/orus-dev/sentryx-server.git"
INSTALL_DIR="$HOME/sentryx-server"
SERVICE_NAME="sentryx-server"
SERVICE_FILE="$HOME/.config/systemd/user/$SERVICE_NAME.service"
CARGO_BIN="$(which cargo)"

if [ -d $INSTALL_DIR ]; then
    mv $INSTALL_DIR $INSTALL_DIR"-backup"
fi

# Step 1: Clone the repo
if [ -d "$INSTALL_DIR" ]; then
    echo "Directory $INSTALL_DIR already exists. Pulling latest changes..."
    git -C "$INSTALL_DIR" pull
else
    echo "Cloning $REPO_URL into $INSTALL_DIR..."
    git clone "$REPO_URL" "$INSTALL_DIR"
fi

# Step 2: Create systemd user service file
mkdir -p "$(dirname "$SERVICE_FILE")"

cat > "$SERVICE_FILE" <<EOF
[Unit]
Description=SentryX Server

[Service]
Type=simple
WorkingDirectory=$INSTALL_DIR
ExecStart=$CARGO_BIN run --release
Restart=on-failure

[Install]
WantedBy=default.target
EOF

# Step 3: Reload systemd, enable and start service
echo "Reloading user systemd daemon..."
systemctl --user daemon-reexec
systemctl --user daemon-reload

echo "Enabling and starting the service..."
systemctl --user enable --now "$SERVICE_NAME.service"

echo "Installation complete. Use 'systemctl --user status $SERVICE_NAME' to check the service status."
