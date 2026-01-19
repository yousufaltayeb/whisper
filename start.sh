#!/bin/bash
# Whisper Startup Script
# Usage: Add the following line to your ~/.xinitrc or WM startup file:
# /home/yousif/TOP/whisper/start.sh &

# Ensure we are in the project directory
cd "$(dirname "$0")"

echo "Starting Whisper..."

# Loop forever to restart if it crashes
while true; do
    # Run using the virtual environment python
    # Redirect stdout/stderr to a log file for debugging
    # -u option forces unbuffered output so logs appear immediately
    .venv/bin/python -u dictate.py >> whisper.log 2>&1
    
    # If it crashes, wait 5 seconds before restarting
    echo "Whisper crashed or exited. Restarting in 5s..." >> whisper.log
    sleep 5
done
