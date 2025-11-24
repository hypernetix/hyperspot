#!/usr/bin/env python3
"""Entrypoint script for the mock HTTP server Docker container."""

import sys
import signal
import time
from pathlib import Path

# Add mock_server module to path
sys.path.insert(0, '/app')

from mock_server import MockHTTPServer


def main():
    """Run the mock HTTP server."""
    # Create and start server
    testdata_dir = Path('/app/testdata')
    server = MockHTTPServer(testdata_dir, host='0.0.0.0', port=8080)
    server.start()
    
    # Setup signal handlers for graceful shutdown
    def shutdown_handler(signum, frame):
        print("Shutting down mock server...", flush=True)
        server.stop()
        sys.exit(0)
    
    signal.signal(signal.SIGTERM, shutdown_handler)
    signal.signal(signal.SIGINT, shutdown_handler)
    
    print("Mock HTTP server ready", flush=True)
    
    # Keep the process running
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        shutdown_handler(None, None)


if __name__ == '__main__':
    main()

