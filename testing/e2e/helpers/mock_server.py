"""
Mock HTTP server for E2E URL-based parsing tests.

This module provides:
- A simple HTTP server that serves static files and provides test endpoints
- Automatic port detection and management
- Docker/local mode detection
- Helper functions to generate mock URLs
"""

import os
import socket
import threading
import time
from pathlib import Path
from http.server import HTTPServer, SimpleHTTPRequestHandler
import json
from typing import Optional


class MockHTTPHandler(SimpleHTTPRequestHandler):
    """Custom HTTP handler for the mock server."""
    
    # Add proper MIME types for document files
    extensions_map = {
        **SimpleHTTPRequestHandler.extensions_map,
        '.docx': 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
        '.xlsx': 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
        '.pptx': 'application/vnd.openxmlformats-officedocument.presentationml.presentation',
        '.pdf': 'application/pdf',
        '.txt': 'text/plain',
        '.md': 'text/markdown',
    }
    
    def __init__(self, *args, mock_data_dir: str, **kwargs):
        self.mock_data_dir = mock_data_dir
        super().__init__(*args, directory=mock_data_dir, **kwargs)
    
    def do_GET(self):
        """Handle GET requests."""
        # Handle /ping endpoint
        if self.path == '/ping' or self.path == '/ping/':
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            response = {"status": "ok", "message": "Mock server is running"}
            self.wfile.write(json.dumps(response).encode('utf-8'))
            return
        
        # Handle /health endpoint
        if self.path == '/health' or self.path == '/health/':
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            response = {"status": "healthy"}
            self.wfile.write(json.dumps(response).encode('utf-8'))
            return
        
        # Serve static files
        super().do_GET()
    
    def log_message(self, format, *args):
        """Suppress logging unless in debug mode."""
        if os.getenv("MOCK_SERVER_DEBUG"):
            super().log_message(format, *args)


class MockHTTPServer:
    """Mock HTTP server for E2E tests."""
    
    def __init__(self, mock_data_dir: Path, host: str = '0.0.0.0', port: int = 0):
        """
        Initialize the mock server.
        
        Args:
            mock_data_dir: Directory containing files to serve
            host: Host to bind to (default: 0.0.0.0 for Docker compatibility)
            port: Port to bind to (default: 0 for auto-detection)
        """
        self.mock_data_dir = mock_data_dir
        self.host = host
        self.port = port
        self.server: Optional[HTTPServer] = None
        self.server_thread: Optional[threading.Thread] = None
        self._actual_port: Optional[int] = None
    
    def start(self):
        """Start the mock HTTP server."""
        if self.server is not None:
            raise RuntimeError("Server is already running")
        
        # Create handler with mock_data_dir
        def handler(*args, **kwargs):
            return MockHTTPHandler(*args, mock_data_dir=str(self.mock_data_dir), **kwargs)
        
        # Create and start server
        self.server = HTTPServer((self.host, self.port), handler)
        self._actual_port = self.server.server_port
        
        # Start server in background thread
        self.server_thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.server_thread.start()
        
        # Wait a bit for server to be ready
        time.sleep(0.1)
        
        print(f"Mock HTTP server started on {self.host}:{self._actual_port}")
    
    def stop(self):
        """Stop the mock HTTP server."""
        if self.server is None:
            return
        
        self.server.shutdown()
        if self.server_thread:
            self.server_thread.join(timeout=5)
        
        self.server = None
        self.server_thread = None
        self._actual_port = None
        
        print("Mock HTTP server stopped")
    
    def get_port(self) -> int:
        """Get the actual port the server is running on."""
        if self._actual_port is None:
            raise RuntimeError("Server is not running")
        return self._actual_port
    
    def get_base_url(self) -> str:
        """Get the base URL for the server."""
        if self._actual_port is None:
            raise RuntimeError("Server is not running")
        return f"http://127.0.0.1:{self._actual_port}"


# Global server instance
_mock_server: Optional[MockHTTPServer] = None
_is_docker_mode: Optional[bool] = None


def is_docker_mode() -> bool:
    """
    Detect if we're running in Docker mode.
    
    In Docker mode, the backend runs in a container and we use docker-compose.
    In local mode, both the backend and tests run on the host machine.
    
    We detect Docker mode by checking if E2E_DOCKER_MODE env var is set,
    which the ci.py script sets when running with --docker flag.
    
    Returns:
        True if running in Docker mode, False otherwise
    """
    global _is_docker_mode
    
    if _is_docker_mode is not None:
        return _is_docker_mode
    
    # Check if E2E_DOCKER_MODE is explicitly set
    _is_docker_mode = os.getenv("E2E_DOCKER_MODE", "").lower() in ("1", "true", "yes")
    
    return _is_docker_mode


def get_mock_base_url() -> str:
    """
    Get the base URL for the mock server.
    
    In Docker mode, this returns the Docker service DNS name.
    In local mode, this returns the localhost URL with dynamic port.
    
    Returns:
        Base URL for the mock server
    """
    if is_docker_mode():
        # In Docker mode, use the service DNS name
        return "http://mock:8080"
    else:
        # In local mode, use the dynamic port
        if _mock_server is None:
            raise RuntimeError("Mock server is not running in local mode")
        return _mock_server.get_base_url()


def mock_url(relative_path: str) -> str:
    """
    Generate a URL for a mock file.
    
    Args:
        relative_path: Relative path from testdata (e.g., "docx/example.docx" or "pdf/file.pdf")
    
    Returns:
        Full URL to the file on the mock server
    """
    base_url = get_mock_base_url()
    return f"{base_url}/{relative_path}"


def start_mock_server(mock_data_dir: Path) -> MockHTTPServer:
    """
    Start the mock HTTP server (local mode only).
    
    Args:
        mock_data_dir: Directory containing files to serve
    
    Returns:
        The started MockHTTPServer instance
    """
    global _mock_server
    
    if is_docker_mode():
        raise RuntimeError("Cannot start local mock server in Docker mode")
    
    if _mock_server is not None:
        raise RuntimeError("Mock server is already running")
    
    _mock_server = MockHTTPServer(mock_data_dir)
    _mock_server.start()
    
    return _mock_server


def stop_mock_server():
    """Stop the mock HTTP server (local mode only)."""
    global _mock_server
    
    if _mock_server is None:
        return
    
    _mock_server.stop()
    _mock_server = None


def get_mock_server() -> Optional[MockHTTPServer]:
    """Get the global mock server instance."""
    return _mock_server

