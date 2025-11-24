#!/usr/bin/env python
import argparse
import os
import subprocess
import sys
import time
from urllib.request import urlopen
from urllib.error import URLError, HTTPError

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PYTHON = sys.executable or "python"


def run_cmd(cmd, env=None):
    print(f"> {' '.join(cmd)}")
    result = subprocess.run(cmd, env=env)
    if result.returncode != 0:
        sys.exit(result.returncode)
    return result


def run_cmd_allow_fail(cmd, env=None):
    print(f"> {' '.join(cmd)}")
    return subprocess.run(cmd, env=env)


def step(msg):
    print(f"\n== {msg}")


def cmd_fmt(args):
    step("Running cargo fmt")
    if args.fix:
        run_cmd(["cargo", "fmt", "--all"])
        print("Code formatted successfully")
    else:
        result = run_cmd_allow_fail(["cargo", "fmt", "--all", "--", "--check"])
        if result.returncode == 0:
            print("Code formatting is correct")
        else:
            print("Formatting issues found. Run: python scripts/ci.py fmt --fix")
            sys.exit(result.returncode)


def cmd_clippy(args):
    step("Running cargo clippy")
    if args.fix:
        run_cmd(["cargo", "clippy", "--workspace", "--all-targets", "--fix", "--allow-dirty"])
        print("Clippy issues fixed")
    else:
        result = run_cmd_allow_fail(
            ["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"]
        )
        if result.returncode == 0:
            print("No clippy warnings found")
        else:
            print("Clippy warnings found. Run: python scripts/ci.py clippy --fix")
            sys.exit(result.returncode)


def cmd_test(_args):
    step("Running cargo test")
    run_cmd(["cargo", "test", "--workspace"])
    print("All tests passed")


def ensure_tool(binary, install_hint=None):
    result = run_cmd_allow_fail([binary, "--version"])
    if result.returncode != 0:
        msg = f"{binary} is not installed"
        if install_hint:
            msg += f". Install with: {install_hint}"
        print(msg)
        sys.exit(1)


def cmd_audit(_args):
    step("Running cargo audit")
    ensure_tool("cargo-audit", "cargo install cargo-audit")
    run_cmd(["cargo", "audit"])
    print("No security vulnerabilities found")


def cmd_deny(_args):
    step("Running cargo deny")
    ensure_tool("cargo-deny", "cargo install cargo-deny")
    run_cmd(["cargo", "deny", "check"])
    print("No licensing or dependency issues found")


def cmd_security(_args):
    step("Running security checks (audit + deny)")
    cmd_audit(_args)
    cmd_deny(_args)
    print("All security checks passed")


def cmd_check(args):
    step("Running full check suite")
    cmd_fmt(args)
    cmd_clippy(args)
    cmd_test(args)
    cmd_security(args)
    print("All checks passed")


def cmd_quickstart(_args):
    step("Starting HyperSpot in quickstart mode")
    data_dir = os.path.join(PROJECT_ROOT, "data")
    if not os.path.isdir(data_dir):
        os.makedirs(data_dir, exist_ok=True)
        print(f"Created data directory: {data_dir}")
    run_cmd(["cargo", "run", "--bin", "hyperspot-server", "--", "--config", "config/quickstart.yaml", "run"])


def wait_for_health(base_url, timeout_secs=30):
    url = f"{base_url.rstrip('/')}/healthz"
    step(f"Waiting for API to be ready at {url}")
    start = time.time()
    while True:
        try:
            with urlopen(url, timeout=1) as resp:
                if 200 <= resp.status < 300:
                    print("API is ready")
                    return
        except (URLError, HTTPError):
            pass

        if time.time() - start > timeout_secs:
            print("ERROR: API did not become ready in time")
            sys.exit(1)
        time.sleep(1)


def check_pytest():
    step("Checking pytest")
    # First try "python -m pytest"
    result = run_cmd_allow_fail([PYTHON, "-m", "pytest", "--version"])
    if result.returncode == 0:
        return
    # Then try "pytest" directly
    result = run_cmd_allow_fail(["pytest", "--version"])
    if result.returncode == 0:
        return
    print("ERROR: pytest is not installed. Install with: pip install -r testing/e2e/requirements.txt")
    sys.exit(1)


def cmd_e2e(args):
    base_url = os.environ.get("E2E_BASE_URL", "http://localhost:8087")
    check_pytest()

    docker_env_started = False

    if args.docker:
        step("Running E2E tests in Docker mode")

        # Check docker
        result = run_cmd_allow_fail(["docker", "version"])
        if result.returncode != 0:
            print("ERROR: docker is not installed or not in PATH")
            sys.exit(1)

        result = run_cmd_allow_fail(["docker", "compose", "version"])
        if result.returncode != 0:
            print("ERROR: 'docker compose' is not available")
            sys.exit(1)

        # Build image
        step("Building Docker image for E2E tests")
        run_cmd(["docker", "build", "-f", "testing/docker/hyperspot.Dockerfile", "-t", "hyperspot-api:e2e", "."])

        # Start environment
        step("Starting E2E docker-compose environment")
        run_cmd(["docker", "compose", "-f", "testing/docker/docker-compose.yml", "up", "-d"])
        docker_env_started = True

        # Wait for healthz
        wait_for_health(base_url)
    else:
        step("Running E2E tests in local mode")
        # Check local server
        try:
            wait_for_health(base_url, timeout_secs=5)
        except SystemExit:
            print("")
            print(f"WARNING: Server not responding at {base_url}/healthz")
            print("")
            print("Please start hyperspot-server before running E2E tests, for example:")
            print("  make example")
            print("  OR")
            print("  cargo run --bin hyperspot-server --features users-info-example -- --config config/quickstart.yaml")
            print("")
            print("To use Docker environment instead, run:")
            print("  python scripts/ci.py e2e --docker")
            print("")
            print("To use a different URL, set E2E_BASE_URL environment variable:")
            print("  E2E_BASE_URL=http://localhost:9000 python scripts/ci.py e2e")
            print("")
            sys.exit(1)

    # Run pytest
    step("Running pytest")
    env = os.environ.copy()
    env["E2E_BASE_URL"] = base_url
    
    # Set E2E_DOCKER_MODE flag for the tests to know which mode they're in
    if args.docker:
        env["E2E_DOCKER_MODE"] = "1"

    pytest_cmd = [PYTHON, "-m", "pytest", "testing/e2e", "-vv"]
    if args.pytest_args:
        pytest_cmd.extend(args.pytest_args)

    result = run_cmd_allow_fail(pytest_cmd, env=env)
    exit_code = result.returncode

    if args.docker and docker_env_started:
        step("Stopping E2E docker-compose environment")
        run_cmd_allow_fail(["docker", "compose", "-f", "testing/docker/docker-compose.yml", "down", "-v"])

    print("")
    if exit_code == 0:
        print("E2E tests passed")
    else:
        print("E2E tests failed")

    sys.exit(exit_code)


def build_parser():
    parser = argparse.ArgumentParser(
        description="HyperSpot CI utility (Python, cross-platform)",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    # fmt
    p_fmt = subparsers.add_parser("fmt", help="Check or fix code formatting")
    p_fmt.add_argument("--fix", action="store_true", help="Auto-format code")
    p_fmt.set_defaults(func=cmd_fmt)

    # clippy
    p_clippy = subparsers.add_parser("clippy", help="Run clippy lints")
    p_clippy.add_argument("--fix", action="store_true", help="Auto-fix clippy issues")
    p_clippy.set_defaults(func=cmd_clippy)

    # test
    p_test = subparsers.add_parser("test", help="Run unit tests")
    p_test.set_defaults(func=cmd_test)

    # audit
    p_audit = subparsers.add_parser("audit", help="Run cargo audit")
    p_audit.set_defaults(func=cmd_audit)

    # deny
    p_deny = subparsers.add_parser("deny", help="Run cargo deny checks")
    p_deny.set_defaults(func=cmd_deny)

    # security
    p_sec = subparsers.add_parser("security", help="Run security checks (audit + deny)")
    p_sec.set_defaults(func=cmd_security)

    # check
    p_check = subparsers.add_parser("check", help="Run full check suite (fmt + clippy + test + security)")
    p_check.add_argument("--fix", action="store_true", help="Auto-fix formatting and clippy issues")
    p_check.set_defaults(func=cmd_check)

    # quickstart
    p_qs = subparsers.add_parser("quickstart", help="Start server in quickstart mode")
    p_qs.set_defaults(func=cmd_quickstart)

    # e2e
    p_e2e = subparsers.add_parser("e2e", help="Run end-to-end tests")
    p_e2e.add_argument(
        "--docker",
        action="store_true",
        help="Run tests in Docker environment instead of local server",
    )
    p_e2e.add_argument(
        "pytest_args",
        nargs=argparse.REMAINDER,
        help="Extra arguments passed to pytest (use -- to separate)",
    )
    p_e2e.set_defaults(func=cmd_e2e)

    return parser


def main():
    os.chdir(PROJECT_ROOT)
    parser = build_parser()
    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
