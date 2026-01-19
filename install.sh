#!/bin/bash
# Install SoupaWhisper on Linux
# Supports: Ubuntu, Pop!_OS, Debian, Fedora, Arch, Void

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_DIR="$HOME/.config/soupawhisper"
SERVICE_DIR="$HOME/.config/systemd/user"
VENV_DIR="$SCRIPT_DIR/.venv"

# Detect package manager
detect_package_manager() {
    if command -v apt &> /dev/null; then
        echo "apt"
    elif command -v dnf &> /dev/null; then
        echo "dnf"
    elif command -v pacman &> /dev/null; then
        echo "pacman"
    elif command -v zypper &> /dev/null; then
        echo "zypper"
    elif command -v xbps-install &> /dev/null; then
        echo "xbps"
    else
        echo "unknown"
    fi
}

# Install system dependencies
install_deps() {
    local pm=$(detect_package_manager)

    echo "Detected package manager: $pm"
    echo "Installing system dependencies..."

    case $pm in
        apt)
            sudo apt update
            sudo apt install -y alsa-utils xclip xdotool libnotify-bin
            ;;
        dnf)
            sudo dnf install -y alsa-utils xclip xdotool libnotify
            ;;
        pacman)
            sudo pacman -S --noconfirm alsa-utils xclip xdotool libnotify
            ;;
        zypper)
            sudo zypper install -y alsa-utils xclip xdotool libnotify-tools
            ;;
        xbps)
            sudo xbps-install -Sy alsa-utils xclip xdotool libnotify
            ;;
        *)
            echo "Unknown package manager. Please install manually:"
            echo "  alsa-utils xclip xdotool libnotify"
            ;;
    esac
}

# Find compatible Python version
find_python_command() {
    # Prefer Python 3.11 (most stable for ML libs currently)
    if command -v python3.11 &> /dev/null; then
        echo "python3.11"
        return
    fi

    # Try 3.12
    if command -v python3.12 &> /dev/null; then
        echo "python3.12"
        return
    fi

    # Try 3.10
    if command -v python3.10 &> /dev/null; then
        echo "python3.10"
        return
    fi
    
    # Fallback
    echo "python3"
}

# Install Python dependencies
install_python() {
    echo ""
    echo "Setting up Python environment..."

    local python_cmd=$(find_python_command)
    echo "Using Python interpreter: $python_cmd"

    # Create venv if it doesn't exist
    if [ ! -d "$VENV_DIR" ]; then
        echo "Creating virtual environment at $VENV_DIR..."
        $python_cmd -m venv "$VENV_DIR"
    fi

    # Install dependencies
    echo "Installing Python dependencies..."
    source "$VENV_DIR/bin/activate"

    if command -v uv &> /dev/null; then
        echo "Using uv..."
        uv pip install -r "$SCRIPT_DIR/requirements.txt"
    else
        echo "Using pip..."
        pip install -r "$SCRIPT_DIR/requirements.txt"
    fi
}

# Setup config file
setup_config() {
    echo ""
    echo "Setting up config..."
    mkdir -p "$CONFIG_DIR"

    if [ ! -f "$CONFIG_DIR/config.ini" ]; then
        cp "$SCRIPT_DIR/config.example.ini" "$CONFIG_DIR/config.ini"
        echo "Created config at $CONFIG_DIR/config.ini"
    else
        echo "Config already exists at $CONFIG_DIR/config.ini"
    fi
}

# Install systemd service
install_systemd_service() {
    echo ""
    echo "Installing systemd user service..."

    mkdir -p "$SERVICE_DIR"

    # Get current display settings
    local display="${DISPLAY:-:0}"
    local xauthority="${XAUTHORITY:-$HOME/.Xauthority}"
    
    cat > "$SERVICE_DIR/soupawhisper.service" << EOF
[Unit]
Description=SoupaWhisper Voice Dictation
After=graphical-session.target

[Service]
Type=simple
WorkingDirectory=$SCRIPT_DIR
ExecStart=$VENV_DIR/bin/python $SCRIPT_DIR/dictate.py
Restart=on-failure
RestartSec=5

# X11 display access
Environment=DISPLAY=$display
Environment=XAUTHORITY=$xauthority

[Install]
WantedBy=default.target
EOF

    echo "Created service at $SERVICE_DIR/soupawhisper.service"

    # Reload and enable
    systemctl --user daemon-reload
    systemctl --user enable soupawhisper

    echo ""
    echo "Service installed! Commands:"
    echo "  systemctl --user start soupawhisper   # Start"
    echo "  systemctl --user stop soupawhisper    # Stop"
    echo "  systemctl --user status soupawhisper  # Status"
    echo "  journalctl --user -u soupawhisper -f  # Logs"
}

# Install runit service (Void Linux)
install_runit_service() {
    echo ""
    echo "Generating runit service configuration..."

    local runit_dir="$SCRIPT_DIR/runit"
    local run_file="$runit_dir/run"
    local current_user=$(whoami)
    
    mkdir -p "$runit_dir"

    # Get current display settings
    local display="${DISPLAY:-:0}"
    local xauthority="${XAUTHORITY:-$HOME/.Xauthority}"

    cat > "$run_file" << EOF
#!/bin/sh
# Runit service script for SoupaWhisper
exec 2>&1

# Set environment
export DISPLAY=$display
export XAUTHORITY=$xauthority

cd "$SCRIPT_DIR"

# Run as user $current_user using chpst (if available) or fallback to direct execution
if command -v chpst >/dev/null; then
    exec chpst -u $current_user .venv/bin/python dictate.py
else
    # Fallback for user-level supervisors
    exec .venv/bin/python dictate.py
fi
EOF

    chmod +x "$run_file"
    
    echo "Service configuration created at: $runit_dir"
    echo ""
    echo "To enable this service:"
    echo "1. System Service (runs on boot, requires sudo):"
    echo "   sudo ln -s $runit_dir /var/service/soupawhisper"
    echo ""
    echo "2. User Service (requires user runsvdir):"
    echo "   ln -s $runit_dir ~/service/soupawhisper"
}

# Main
main() {
    echo "==================================="
    echo "  SoupaWhisper Installer"
    echo "==================================="
    echo ""

    install_deps
    install_python
    setup_config

    echo ""
    local pm=$(detect_package_manager)
    
    if [ "$pm" == "xbps" ]; then
        read -p "Generate runit service configuration? [y/N] " -n 1 -r
        echo ""
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            install_runit_service
        fi
    else
        read -p "Install as systemd service? [y/N] " -n 1 -r
        echo ""
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            install_systemd_service
        fi
    fi

    echo ""
    echo "==================================="
    echo "  Installation complete!"
    echo "==================================="
    echo ""
    echo "To run manually:"
    echo "  source .venv/bin/activate"
    echo "  python dictate.py"
    echo ""
    echo "Config: $CONFIG_DIR/config.ini"
    echo "Hotkey: F12 (hold to record)"
    echo "Exit:   Ctrl+C"
}

main "$@"
