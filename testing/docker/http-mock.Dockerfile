FROM python:3.11-slim

WORKDIR /app

# Copy testdata and server files
COPY ../e2e/testdata /app/testdata
COPY ../e2e/helpers/mock_server.py /app/mock_server.py
COPY ../e2e/helpers/run_mock_server.py /app/run_mock_server.py

# Expose port 8080
EXPOSE 8080

# Run the mock server
CMD ["python", "-u", "/app/run_mock_server.py"]

