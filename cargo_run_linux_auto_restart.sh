#!/bin/bash

# Auto-restart wrapper for control server
# Restarts the server automatically when it exits with code 2 (EtherCAT connection loss)

while true; do
    echo "$(date '+%Y-%m-%d %H:%M:%S') - Starting server..."
    
    # Run the server
    ./cargo_run_linux.sh
    EXIT_CODE=$?
    
    # Check exit code
    if [ $EXIT_CODE -eq 2 ]; then
        echo "$(date '+%Y-%m-%d %H:%M:%S') - Server exited due to EtherCAT connection loss (exit code $EXIT_CODE)"
        echo "$(date '+%Y-%m-%d %H:%M:%S') - Waiting 2 seconds before restart..."
        sleep 2
        echo "$(date '+%Y-%m-%d %H:%M:%S') - Restarting server..."
    elif [ $EXIT_CODE -eq 0 ]; then
        echo "$(date '+%Y-%m-%d %H:%M:%S') - Server exited cleanly (exit code $EXIT_CODE)"
        break
    else
        echo "$(date '+%Y-%m-%d %H:%M:%S') - Server crashed with exit code $EXIT_CODE"
        echo "$(date '+%Y-%m-%d %H:%M:%S') - Waiting 5 seconds before restart..."
        sleep 5
    fi
done

echo "$(date '+%Y-%m-%d %H:%M:%S') - Auto-restart wrapper exiting"
