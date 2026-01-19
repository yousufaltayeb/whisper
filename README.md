# Whisper

A voice dictation tool for Linux using [faster-whisper](https://github.com/SYSTRAN/faster-whisper). 

**Fork Note:** This is an enhanced fork of [soupawhisper](https://github.com/ksred/soupawhisper), expanded to support **Void Linux**, **Runit**, and **PulseAudio/PipeWire**. It also features a **toggle recording** behavior (press to start, press to stop) instead of the original push-to-talk.

## Features

- **Toggle Recording:** Press the hotkey to start recording, press again to stop and transcribe.
- **Fast Transcription:** Uses `faster-whisper` for high-performance inference.
- **Auto-Type:** Automatically copies text to clipboard and types it into the active window.
- **Void Linux Support:** First-class support for Void Linux and Runit service supervision.
- **Notifications:** Desktop notifications for recording status and errors.

## Requirements

- Python 3.10+
- **Audio Backend:** PulseAudio or PipeWire (requires `parecord`)
- **System Tools:** `xclip`, `xdotool`, `libnotify`
- **Linux:** Tested on Void Linux, Ubuntu, Fedora, Arch.

## Installation

### Automatic Installation (Recommended)

The included installer detects your distro and package manager to set everything up, including system dependencies and the Python environment.

```bash
git clone https://github.com/yourusername/whisper.git
cd whisper
chmod +x install.sh
./install.sh
```

### Manual Installation

#### 1. Install System Dependencies

**Void Linux:**
```bash
sudo xbps-install -S pulseaudio-utils xclip xdotool libnotify
```

**Ubuntu / Debian:**
```bash
sudo apt install pulseaudio-utils xclip xdotool libnotify-bin
```

**Fedora:**
```bash
sudo dnf install pulseaudio-utils xclip xdotool libnotify
```

**Arch Linux:**
```bash
sudo pacman -S pulseaudio-utils xclip xdotool libnotify
```

#### 2. Install Python Dependencies

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

#### 3. Setup Config

```bash
mkdir -p ~/.config/whisper
cp config.example.ini ~/.config/whisper/config.ini
```

## Configuration

Edit `~/.config/whisper/config.ini`:

```ini
[whisper]
# Model size: tiny.en, base.en, small.en, medium.en, large-v3
model = base.en

# Device: cpu or cuda (requires cuDNN 9+)
device = cpu

# Compute type: int8 for CPU, float16 for GPU
compute_type = int8

[hotkey]
# Hotkey to toggle recording (default: Alt+O)
# Examples: <alt>+o, <ctrl>+space, f12
key = <alt>+o

[behavior]
# Type text into active input field
auto_type = true

# Show desktop notification
notifications = true
```

## Usage

Start the application manually:

```bash
source .venv/bin/activate
python dictate.py
```

- **Toggle Recording:** Press **Alt+O** (or your configured key) to start recording.
- **Stop & Transcribe:** Press the key again to stop. The text will be copied to your clipboard and typed into the active window.
- **Quit:** Press **Ctrl+C** in the terminal.

## Auto-Start with .xinitrc

To start Whisper automatically when you log in, add the following line to your `~/.xinitrc` file (or your window manager's startup script). 

Make sure to use the **absolute path** to where you cloned the repository.

```bash
# Start Whisper (adjust path as needed)
/path/to/whisper/start.sh &
```

The `start.sh` script handles:
1.  Activating the virtual environment
2.  Logging output to `whisper.log`
3.  Automatically restarting the application if it crashes

## GPU Support (Optional)

To use NVIDIA GPU acceleration:

1.  Install **cuDNN 9** for CUDA 12.
2.  Update `config.ini`:
    ```ini
    device = cuda
    compute_type = float16
    ```
