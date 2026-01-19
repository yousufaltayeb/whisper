#!/bin/bash
# SoupaWhisper Startup Script
# Usage: Add the following line to your ~/.xinitrc or WM startup file:
# /home/yousif/TOP/soupawhisper/start.sh &

# Ensure we are in the project directory
cd "$(dirname "$0")"

echo "Starting SoupaWhisper..."

# Loop forever to restart if it crashes
while true; do
    # Run using the virtual environment python
    # Redirect stdout/stderr to a log file for debugging
    # -u option forces unbuffered output so logs appear immediately
    .venv/bin/python -u dictate.py >> soupawhisper.log 2>&1
    
    # If it crashes, wait 5 seconds before restarting
    echo "SoupaWhisper crashed or exited. Restarting in 5s..." >> soupawhisper.log
    sleep 5
done
