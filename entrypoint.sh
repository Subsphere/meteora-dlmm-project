#!/bin/bash

# Function to handle termination signals
terminate() {
    echo "Terminating processes..."
    pkill -SIGTERM -P $$
    wait
    exit 0
}

# Trap termination signals
trap terminate SIGTERM SIGINT

# Start load_database.py in the background
python load_database.py &

# Start the Rust Web UI
./web-ui/target/release/web-ui &

# Wait for any process to exit
wait -n

# Exit with status of the process that exited first
exit $?
