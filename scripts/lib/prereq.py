#!/usr/bin/env python3
"""
Prerequisite checking module for HyperSpot testing environment.

This module provides classes to check various prerequisites needed for testing,
including services, tools, and dependencies.
"""

import subprocess
import logging
import json
import requests
import time

SERVER_PORT = 8087
BASE_URL = "http://localhost:%d" % SERVER_PORT

PRECHECK_OK = "OK"
PRECHECK_WARNING = "WARNING"
PRECHECK_ERROR = "ERROR"


class Prereq:
    def __init__(self):
        self.name = ""
        self.remediation = ""

    def __str__(self):
        return self.name

    def check(self) -> str:
        return PRECHECK_OK


class PrereqOllama(Prereq):
    def __init__(self):
        super().__init__()
        self.name = "ollama service is running"
        self.remediation = (
            "Install Ollama from https://ollama.ai/download and run "
            "'ollama serve' to start the service"
        )

    def check(self):
        # Check if ollama is installed
        try:
            subprocess.check_output(
                ["ollama", "list"], stderr=subprocess.DEVNULL
            )
        except subprocess.CalledProcessError:
            logging.error("ollama service is not running")
            logging.error(f"Remediation: {self.remediation}")
            return PRECHECK_ERROR
        return PRECHECK_OK


class PrereqOllamaQwen2505b(PrereqOllama):
    def __init__(self):
        super().__init__()
        self.name = "ollama model 'qwen2.5:0.5b' is running"
        self.remediation = (
            "Install the qwen2.5:0.5b model by running "
            "'ollama run qwen2.5:0.5b'"
        )

    def check(self) -> str:
        if not super().check():
            return False

        # Check if qwen2.5:0.5b is installed
        failed = False
        try:
            output = subprocess.check_output(
                ["ollama", "list"], text=True, stderr=subprocess.DEVNULL
            )
            if "qwen2.5:0.5b" not in output:
                failed = True
        except subprocess.CalledProcessError:
            logging.error("ollama service is not running")
            logging.error(f"Possible remediation: {self.remediation}")
            failed = True

        if failed:
            logging.error(
                "ollama model 'qwen2.5:0.5b' is not installed"
            )
            logging.error(f"Possible remediation: {self.remediation}")
            return PRECHECK_ERROR

        return PRECHECK_OK


class PrereqLMStudio(Prereq):
    def __init__(self):
        super().__init__()
        self.name = "LM Studio service is running"
        self.remediation = (
            "Install LM Studio from https://lmstudio.ai/, download a "
            "qwen2.5 0.5B model, and start the local server"
        )

    def check(self) -> str:
        # Check if LM Studio is running by testing its API endpoints
        # LM Studio typically runs on these ports
        ports = [1234, 12345]

        for port in ports:
            try:
                url = f"http://localhost:{port}/v1/models"
                response = requests.get(url, timeout=5)
                if response.status_code == 200:
                    logging.info(
                        f"LM Studio found running on port {port}"
                    )
                    return PRECHECK_OK
            except (requests.exceptions.ConnectionError,
                    requests.exceptions.Timeout):
                continue
            except Exception as e:
                logging.debug(
                    f"Error checking LM Studio on port {port}: {e}"
                )
                continue

        logging.error(
            "LM Studio is not running on any of the expected "
            "ports (1234, 12345)"
        )
        logging.error(f"Possible remediation: {self.remediation}")
        return PRECHECK_ERROR


class PrereqHSSrvMock(Prereq):
    def __init__(self):
        super().__init__()
        self.name = "HyperSpot server is running with mock mode enabled"
        self.remediation = (
            "Start the HyperSpot server with mock mode enabled "
            "(use -mock option)"
        )

    def check(self) -> bool:
        status = PRECHECK_OK

        # Retry configuration
        max_retry_time = 3.0  # 3 seconds total
        base_delay = 0.1  # 100ms initial delay
        multiplier = 2
        timeout_per_request = 2.0  # 2 seconds per request

        start_time = time.time()
        attempt = 0

        while time.time() - start_time < max_retry_time:
            try:
                url = f"{BASE_URL}/llm/services"
                response = requests.get(url, timeout=timeout_per_request)

                if response.status_code != 200:
                    logging.debug(
                        f"HyperSpot server responded with status code: "
                        f"{response.status_code} (attempt {attempt + 1})"
                    )
                else:
                    # Parse JSON response
                    try:
                        services_data = response.json()
                        services = services_data.get('services', [])

                        # Check if mock service exists
                        mock_service_found = False
                        for service in services:
                            if service.get('name') == 'mock':
                                mock_service_found = True
                                logging.info(
                                    "HyperSpot server is running with "
                                    "mock mode enabled"
                                )
                                break

                        if mock_service_found:
                            return PRECHECK_OK
                        else:
                            logging.debug(
                                "HyperSpot server is running but mock mode "
                                f"is not enabled (attempt {attempt + 1})"
                            )
                            status = PRECHECK_WARNING

                    except (json.JSONDecodeError, KeyError) as e:
                        logging.debug(
                            f"Failed to parse server response: {e} "
                            f"(attempt {attempt + 1})"
                        )
                        status = PRECHECK_ERROR

            except (requests.exceptions.ConnectionError,
                    requests.exceptions.Timeout) as e:
                logging.debug(
                    f"Connection error to HyperSpot server: {e} "
                    f"(attempt {attempt + 1})"
                )
                status = PRECHECK_WARNING
            except Exception as e:
                logging.debug(
                    f"Error checking HyperSpot server: {e} "
                    f"(attempt {attempt + 1})"
                )
                status = PRECHECK_WARNING

            # Check if we have time for another retry
            elapsed = time.time() - start_time
            if elapsed >= max_retry_time:
                break

            # Calculate delay for next attempt
            delay = base_delay * (multiplier ** attempt)
            remaining_time = max_retry_time - elapsed

            # Don't wait longer than remaining time
            if delay > remaining_time:
                break

            logging.debug(f"Retrying in {delay:.1f}s... "
                          f"(attempt {attempt + 1})")
            time.sleep(delay)
            attempt += 1

        # All retries exhausted, log final status
        if status == PRECHECK_WARNING:
            if attempt > 0:
                logging.warning(
                    f"Cannot connect to HyperSpot server at {BASE_URL} "
                    f"after {attempt + 1} attempts over "
                    f"{time.time() - start_time:.1f}s"
                )
            else:
                logging.warning(
                    f"Cannot connect to HyperSpot server at {BASE_URL}"
                )
        elif status == PRECHECK_ERROR:
            logging.error(
                f"HyperSpot server communication failed after {attempt + 1} attempts"
            )

        if status != PRECHECK_OK:
            logging.warning(f"Possible remediation: {self.remediation}")

        return status


class PrereqDocker(Prereq):
    def __init__(self):
        super().__init__()
        self.name = "Docker is running"
        self.remediation = (
            "Install Docker from https://docs.docker.com/"
            "get-docker/ and run it"
        )

    def check(self) -> str:
        try:
            subprocess.check_output(
                ["docker", "ps"], stderr=subprocess.DEVNULL
            )
        except subprocess.CalledProcessError:
            logging.error(
                "Docker is not running"
            )
            logging.error(f"Possible remediation: {self.remediation}")
            return PRECHECK_ERROR
        return PRECHECK_OK


class PrereqDredd(Prereq):
    def __init__(self):
        super().__init__()
        self.name = "dredd API testing tool is installed"
        self.remediation = (
            "Install dredd using 'npm install -g dredd' "
            "(requires Node.js and npm)"
        )

    def check(self) -> str:
        # Check if dredd is installed
        try:
            subprocess.check_output(
                ["dredd", "--version"], stderr=subprocess.DEVNULL
            )
            return PRECHECK_OK
        except subprocess.CalledProcessError:
            logging.error(
                "The 'dredd' tool is not installed or not working properly"
            )
            logging.error(f"Possible remediation: {self.remediation}")
            return PRECHECK_ERROR
        except FileNotFoundError:
            logging.error(
                "'dredd' command not found"
            )
            logging.error(f"Possible remediation: {self.remediation}")
            return PRECHECK_ERROR


class PrereqRustPackages(Prereq):
    def __init__(self):
        super().__init__()
        self.name = "required Rust packages are installed"
        self.remediation = (
            "Install missing Rust packages using: "
            "cargo install cargo-llvm-cov"
        )

    def check(self) -> str:
        # Check for essential Rust packages needed for testing
        required_packages = {
            'cargo-llvm-cov': 'cargo llvm-cov --version',
        }

        missing_packages = []

        for package, check_cmd in required_packages.items():
            try:
                # Split the command and run it
                cmd_parts = check_cmd.split()
                subprocess.check_output(cmd_parts, stderr=subprocess.DEVNULL)
            except (subprocess.CalledProcessError, FileNotFoundError):
                missing_packages.append(package)

        if missing_packages:
            logging.error(
                f"Missing required Rust packages: "
                f"{', '.join(missing_packages)}"
            )
            logging.error(f"Possible remediation: {self.remediation}")
            return PRECHECK_ERROR

        return PRECHECK_OK


class PrereqNpm(Prereq):
    def __init__(self):
        super().__init__()
        self.name = "npm is installed"
        self.remediation = (
            "Install Node.js and npm from https://nodejs.org/ "
            "or using a package manager like Homebrew: "
            "'brew install node'"
        )

    def check(self):
        # Check if npm is installed
        try:
            subprocess.check_output(
                ["npm", "--version"], stderr=subprocess.DEVNULL
            )
            return PRECHECK_OK
        except subprocess.CalledProcessError:
            logging.error(
                "npm is not installed or not working properly"
            )
            logging.error(f"Possible remediation: {self.remediation}")
            return PRECHECK_ERROR
        except FileNotFoundError:
            logging.error(
                "'npm' command not found"
            )
            logging.error(f"Possible remediation: {self.remediation}")
            return PRECHECK_ERROR


class PrereqCargo(Prereq):
    def __init__(self):
        super().__init__()
        self.name = "cargo is installed"
        self.remediation = (
            "Install Rust and cargo from https://rustup.rs/ "
            "or using a package manager like Homebrew: "
            "'brew install rust'"
        )

    def check(self):
        # Check if cargo is installed
        try:
            subprocess.check_output(
                ["cargo", "--version"], stderr=subprocess.DEVNULL
            )
            return PRECHECK_OK
        except subprocess.CalledProcessError:
            logging.error(
                "cargo is not installed or not working properly"
            )
            logging.error(f"Possible remediation: {self.remediation}")
            return PRECHECK_ERROR
        except FileNotFoundError:
            logging.error(
                "'cargo' command not found"
            )
            logging.error(f"Possible remediation: {self.remediation}")
            return PRECHECK_ERROR


class PrereqPython(Prereq):
    def __init__(self):
        super().__init__()
        self.name = "python3 is installed"
        self.remediation = (
            "Install Python 3 from https://python.org/ "
            "or using a package manager like Homebrew: "
            "'brew install python3'"
        )

    def check(self):
        # Check if python3 is installed
        try:
            subprocess.check_output(
                ["python3", "--version"], stderr=subprocess.DEVNULL
            )
            return PRECHECK_OK
        except subprocess.CalledProcessError:
            logging.error(
                "python3 is not installed or not working properly"
            )
            logging.error(f"Possible remediation: {self.remediation}")
            return PRECHECK_ERROR
        except FileNotFoundError:
            logging.error(
                "'python3' command not found"
            )
            logging.error(f"Possible remediation: {self.remediation}")
            return PRECHECK_ERROR


class PrereqPytest(Prereq):
    def __init__(self):
        super().__init__()
        self.name = "pytest is installed"
        self.remediation = (
            "Install pytest using 'pip install pytest' "
            "or 'pip install -r testing/e2e/requirements.txt'"
        )

    def check(self):
        # Check if pytest is installed
        try:
            subprocess.check_output(
                ["pytest", "--version"], stderr=subprocess.DEVNULL
            )
            return PRECHECK_OK
        except subprocess.CalledProcessError:
            logging.error(
                "pytest is not installed or not working properly"
            )
            logging.error(f"Possible remediation: {self.remediation}")
            return PRECHECK_ERROR
        except FileNotFoundError:
            logging.error(
                "'pytest' command not found"
            )
            logging.error(f"Possible remediation: {self.remediation}")
            return PRECHECK_ERROR


class PrereqCargoLlvmCov(Prereq):
    def __init__(self):
        super().__init__()
        self.name = "cargo-llvm-cov is installed"
        self.remediation = (
            "Install cargo-llvm-cov using "
            "'cargo install cargo-llvm-cov'"
        )

    def check(self):
        # Check if cargo-llvm-cov is installed
        try:
            subprocess.check_output(
                ["cargo", "llvm-cov", "--version"], stderr=subprocess.DEVNULL
            )
            return PRECHECK_OK
        except subprocess.CalledProcessError:
            logging.error(
                "cargo-llvm-cov is not installed or not working properly"
            )
            logging.error(f"Possible remediation: {self.remediation}")
            return PRECHECK_ERROR
        except FileNotFoundError:
            logging.error(
                "'cargo-llvm-cov' command not found"
            )
            logging.error(f"Possible remediation: {self.remediation}")
            return PRECHECK_ERROR


# Core prerequisites needed for basic testing
CORE_PREREQS = [
    PrereqCargo,
    PrereqPython,
    PrereqPytest,
    PrereqCargoLlvmCov,
]

# E2E testing prerequisites
E2E_LOCAL_PREREQS = [
    PrereqDocker,
    PrereqHSSrvMock,
] + CORE_PREREQS

# Full testing prerequisites
ALL_PREREQS = [
    PrereqDocker,
    PrereqHSSrvMock,
    PrereqDredd,
    PrereqLMStudio,
    PrereqNpm,
    PrereqCargo,
    PrereqOllama,
    PrereqOllamaQwen2505b,
    PrereqPython,
    PrereqPytest,
    PrereqCargoLlvmCov,
    PrereqRustPackages,
] + CORE_PREREQS


def check_prerequisites(prereq_list=None):
    """
    Check a list of prerequisites and return results.

    Args:
        prereq_list: List of prerequisite classes to check.
        Defaults to ALL_PREREQS.

    Returns:
        tuple: (passed_count, total_count, failed_prereqs)
    """
    if prereq_list is None:
        prereq_list = ALL_PREREQS

    passed = 0
    total = len(prereq_list)
    failed_prereqs = []

    # Configure logging to suppress debug/info messages during checks
    logging.basicConfig(
        level=logging.ERROR, format='%(levelname)s: %(message)s'
    )

    for prereq_class in prereq_list:
        prereq = prereq_class()

        try:
            # Temporarily suppress logging for individual checks
            old_level = logging.getLogger().level
            logging.getLogger().setLevel(logging.CRITICAL)

            result = prereq.check()

            # Restore logging level
            logging.getLogger().setLevel(old_level)

            if result in [PRECHECK_OK, PRECHECK_WARNING]:
                passed += 1
            else:
                failed_prereqs.append(prereq)

        except Exception as e:
            logging.error(
                f"Error checking {prereq.name}: {str(e)}"
            )
            failed_prereqs.append(prereq)

    return passed, total, failed_prereqs


def check_environment_ready(env_type="full"):
    """
    Validate that the environment has the necessary
    prerequisites for the given command.

    Args:
        env_type: Type of environment to check ('core', 'e2e', 'full')

    Returns:
        bool: True if environment is ready, False otherwise
    """
    if env_type == "core":
        prereq_list = CORE_PREREQS
    elif env_type == "e2e-local":
        prereq_list = E2E_LOCAL_PREREQS
    else:
        prereq_list = ALL_PREREQS

    passed, total, failed_prereqs = check_prerequisites(prereq_list)

    if failed_prereqs:
        print(f"Environment not ready for {env_type} testing. "
              f"Failed prerequisites:")
        for prereq in failed_prereqs:
            print(f"  - {prereq.name}: {prereq.remediation}")
        return False

    print(f"Environment is ready for {env_type} testing!")
    return True
