#!/usr/bin/env python3
"""
Code coverage collection script for HyperSpot.
Supports unit tests, e2e tests, and combined coverage.
"""
import argparse
import json
import os
import signal
import socket
import subprocess
import sys
import time
from pathlib import Path
from typing import Optional
from urllib.request import urlopen
from urllib.error import URLError, HTTPError

import yaml

# Import prereq module for environment validation
from lib.prereq import check_environment_ready

PROJECT_ROOT = Path(__file__).parent.parent.absolute()
COVERAGE_DIR = PROJECT_ROOT / "coverage"
PYTHON = sys.executable or "python3"
COVERAGE_THRESHOLD = 70

FILE_PATH_COL_WIDTH = 70
COVERAGE_CELL_COL_WIDTH = 18
SEPARATOR_WIDTH = FILE_PATH_COL_WIDTH + COVERAGE_CELL_COL_WIDTH * 3


def run_cmd(cmd, env=None, cwd=None):
    """Run a command and exit on failure."""
    print(f"> {' '.join(str(c) for c in cmd)}")
    result = subprocess.run(cmd, env=env, cwd=cwd)
    if result.returncode != 0:
        sys.exit(result.returncode)
    return result


def run_cmd_allow_fail(cmd, env=None, cwd=None):
    """Run a command and return result without exiting."""
    print(f"> {' '.join(str(c) for c in cmd)}")
    return subprocess.run(cmd, env=env, cwd=cwd)


def run_cmd_capture(cmd, env=None, cwd=None):
    """Run a command and capture output."""
    print(f"> {' '.join(str(c) for c in cmd)}")
    result = subprocess.run(
        cmd, env=env, cwd=cwd, capture_output=True, text=True
    )
    if result.returncode != 0:
        print(result.stderr, file=sys.stderr)
        sys.exit(result.returncode)
    return result


def step(msg):
    """Print a step message."""
    print(f"\n{'='*SEPARATOR_WIDTH}")
    print(f"  {msg}")
    print(f"{'='*SEPARATOR_WIDTH}\n")


def ensure_tool(binary, install_hint=None):
    """Ensure a tool is installed."""
    # Special handling for cargo-llvm-cov
    if binary == "cargo-llvm-cov":
        result = run_cmd_allow_fail(["cargo", "llvm-cov", "--version"])
    else:
        result = run_cmd_allow_fail([binary, "--version"])

    if result.returncode != 0:
        msg = f"{binary} is not installed"
        if install_hint:
            msg += f". Install with: {install_hint}"
        print(msg, file=sys.stderr)
        sys.exit(1)


def wait_for_health(base_url, timeout_secs=30, log_path: Optional[Path] = None):
    """Wait for the server to be healthy.

    Tries both /healthz and /health. On timeout, prints tail of server logs
    if a log_path is provided.
    """
    paths = ["/healthz", "/health"]
    step(
        f"Waiting for API to be ready at {base_url} (paths: {', '.join(paths)})"
    )
    start = time.time()
    while True:
        for p in paths:
            url = f"{base_url.rstrip('/')}{p}"
            try:
                with urlopen(url, timeout=2) as resp:
                    if 200 <= resp.status < 300:
                        print("OK. API is ready")
                        return
            except (URLError, HTTPError):
                pass

        if time.time() - start > timeout_secs:
            print("ERROR: API did not become ready in time", file=sys.stderr)
            # Best-effort: print last lines of server logs to help debugging
            if log_path and Path(log_path).exists():
                try:
                    print("\n--- Server log (tail) ---")
                    lines = Path(log_path).read_text(
                        encoding="utf-8", errors="ignore"
                    ).splitlines()
                    tail = lines[-200:]
                    for line in tail:
                        print(line)
                    print("--- End server log ---\n")
                except Exception:
                    pass
            sys.exit(1)
        time.sleep(1)


def wait_for_tcp(host: str, port: int, timeout_secs: int = 30, log_path: Optional[Path] = None):
    """Wait until a TCP port is accepting connections."""
    step(f"Waiting for TCP {host}:{port} to accept connections")
    start = time.time()
    while True:
        try:
            with socket.create_connection((host, port), timeout=1.0):
                print("OK. TCP port is accepting connections")
                return
        except OSError:
            pass
        if time.time() - start > timeout_secs:
            print("ERROR: TCP port did not become ready in time", file=sys.stderr)
            # Best-effort: print server log tail
            if log_path and Path(log_path).exists():
                try:
                    print("\n--- Server log (tail) ---")
                    lines = Path(log_path).read_text(
                        encoding="utf-8", errors="ignore"
                    ).splitlines()
                    for line in lines[-200:]:
                        print(line)
                    print("--- End server log ---\n")
                except Exception:
                    pass
            sys.exit(1)
        time.sleep(0.5)


def supports_color():
    """Check if terminal supports color output."""
    if not sys.stdout.isatty():
        return False

    # Check COLORTERM variable
    colorterm = os.environ.get('COLORTERM', '')
    if colorterm in ('truecolor', '24bit'):
        return True

    # Check TERM variable
    term = os.environ.get('TERM', '')
    if 'color' in term:
        return True

    return False


def format_coverage_cell(covered, total,
                         threshold=COVERAGE_THRESHOLD,
                         cell_width=COVERAGE_CELL_COL_WIDTH,
                         use_color=False):
    """
    Format coverage cell as '%3d %% (missed)' with optional color coding.
    """
    missed = total - covered
    if total == 0:
        percent = 0
    else:
        percent = int(round((covered / total) * 100.0))

    warning = "!" if percent < threshold else " "
    cell_text = f"{warning}{percent:3d} % ({missed})"

    if use_color and supports_color():
        if percent < threshold:
            # Light red for below threshold
            return f"\033[91m{cell_text:<{cell_width}}\033[0m"
        else:
            # Light green for above threshold
            return f"\033[92m{cell_text:<{cell_width}}\033[0m"
    else:
        return cell_text.ljust(cell_width)


def make_relative_path(filepath, project_root):
    """Normalize coverage paths to project-relative.

    - Supports absolute and remapped paths.
    - Strips leading './' if present.
    """
    root = Path(project_root)
    s = str(filepath).replace("\\", "/")
    if s.startswith("./"):
        s = s[2:]
    # If it's already a libs/ or modules/ or apps/ relative path, accept it
    if s.startswith("libs/") or s.startswith("modules/") or s.startswith("apps/"):
        return s
    # Otherwise try to relativize
    try:
        return str(Path(s).relative_to(root))
    except Exception:
        return s


def categorize_file(rel_path):
    """Categorize file into: 'file', 'module', 'lib', or 'external'."""
    # Normalize leading './'
    rel_path = rel_path[2:] if rel_path.startswith('./') else rel_path
    if rel_path.startswith('libs/'):
        # Extract lib name: libs/modkit-db/... -> modkit-db
        parts = rel_path.split('/')
        if len(parts) >= 2:
            return 'lib', parts[1]
        return 'file', None
    elif rel_path.startswith('modules/'):
        # Extract module name: modules/api_ingress/... -> api_ingress
        parts = rel_path.split('/')
        if len(parts) >= 2:
            return 'module', parts[1]
        return 'file', None
    elif rel_path.startswith('apps/'):
        # Apps are individual files
        return 'file', None
    else:
        # External dependencies
        return 'external', None


def enumerate_project_rs_files(project_root):
    """List Rust source files under libs/ and modules/ as relative paths."""
    root = Path(project_root)
    rels = []
    for top in [root / "libs", root / "modules"]:
        if not top.exists():
            continue
        for p in top.rglob("*.rs"):
            try:
                rels.append(str(p.relative_to(root)))
            except ValueError:
                rels.append(str(p))
    return rels


def count_non_empty_lines(abs_path):
    """Approximate LOC by counting non-empty lines for a file."""
    try:
        with open(abs_path, "r", encoding="utf-8", errors="ignore") as f:
            return sum(1 for ln in f if ln.strip())
    except FileNotFoundError:
        return 0


def aggregate_coverage_data(files_data, project_root):
    """
    Aggregate coverage data by files, modules, and libs.
    Returns: (individual_files, aggregated_groups, total)
    """
    individual_files = []
    groups = {}  # module/lib name -> aggregated stats
    total_stats = {
        'regions': {'covered': 0, 'total': 0},
        'functions': {'covered': 0, 'total': 0},
        'lines': {'covered': 0, 'total': 0}
    }

    for file_data in files_data:
        filepath = file_data['filename']
        rel_path = make_relative_path(filepath, project_root)
        category, group_name = categorize_file(rel_path)

        # Skip external files
        if category == 'external':
            continue

        # Extract summary stats
        summary = file_data.get('summary', {})
        file_stats = {
            'path': rel_path,
            'regions': {
                'covered': summary.get('regions', {}).get('covered', 0),
                'total': summary.get('regions', {}).get('count', 0)
            },
            'functions': {
                'covered': summary.get('functions', {}).get('covered', 0),
                'total': summary.get('functions', {}).get('count', 0)
            },
            'lines': {
                'covered': summary.get('lines', {}).get('covered', 0),
                'total': summary.get('lines', {}).get('count', 0)
            }
        }

        # Add to individual files
        individual_files.append(file_stats)

        # Aggregate into groups (modules/libs)
        if category in ['module', 'lib'] and group_name:
            if group_name not in groups:
                groups[group_name] = {
                    'name': group_name,
                    'type': category,
                    'regions': {'covered': 0, 'total': 0},
                    'functions': {'covered': 0, 'total': 0},
                    'lines': {'covered': 0, 'total': 0}
                }
            for metric in ['regions', 'functions', 'lines']:
                groups[group_name][metric]['covered'] += (
                    file_stats[metric]['covered']
                )
                groups[group_name][metric]['total'] += (
                    file_stats[metric]['total']
                )

        # Aggregate into total
        for metric in ['regions', 'functions', 'lines']:
            total_stats[metric]['covered'] += file_stats[metric]['covered']
            total_stats[metric]['total'] += file_stats[metric]['total']

    return individual_files, list(groups.values()), total_stats


def format_coverage_row(
    name, stats, max_name_len=FILE_PATH_COL_WIDTH, threshold=COVERAGE_THRESHOLD, use_color=False
):
    """Format a single coverage row with name and all metrics."""
    # Truncate name if too long
    if len(name) > max_name_len - 2:
        name = "..." + name[-(max_name_len - 3):]

    cell_width = COVERAGE_CELL_COL_WIDTH

    # Format all coverage cells
    reg_cov = format_coverage_cell(
        stats['regions']['covered'],
        stats['regions']['total'],
        threshold,
        cell_width,
        use_color
    )
    func_cov = format_coverage_cell(
        stats['functions']['covered'],
        stats['functions']['total'],
        threshold,
        cell_width,
        use_color
    )
    line_cov = format_coverage_cell(
        stats['lines']['covered'],
        stats['lines']['total'],
        threshold,
        cell_width,
        use_color
    )

    return (
        f"{name:<{max_name_len}} "
        f"{reg_cov} "
        f"{func_cov} "
        f"{line_cov}"
    )


def format_section_header(title, separator="-"):
    """Format a section header with title and 3-line column headers."""
    header_line1 = (
        f"{title:<{FILE_PATH_COL_WIDTH}} "
        f"{'Regions':<{COVERAGE_CELL_COL_WIDTH}} "
        f"{'Functions':<{COVERAGE_CELL_COL_WIDTH}} "
        f"{'Lines':<{COVERAGE_CELL_COL_WIDTH}}"
    )
    header_line2 = (
        f"{'':<{FILE_PATH_COL_WIDTH}} "
        f"{'Coverage %':<{COVERAGE_CELL_COL_WIDTH}} "
        f"{'Coverage %':<{COVERAGE_CELL_COL_WIDTH}} "
        f"{'Coverage %':<{COVERAGE_CELL_COL_WIDTH}}"
    )
    header_line3 = (
        f"{'':<{FILE_PATH_COL_WIDTH}} "
        f"{'(missed)':<{COVERAGE_CELL_COL_WIDTH}} "
        f"{'(missed)':<{COVERAGE_CELL_COL_WIDTH}} "
        f"{'(missed)':<{COVERAGE_CELL_COL_WIDTH}}"
    )

    return [
        separator * SEPARATOR_WIDTH,
        header_line1,
        header_line2,
        header_line3,
        separator * SEPARATOR_WIDTH
    ]


def format_custom_coverage_report(json_data,
                                  project_root,
                                  threshold=COVERAGE_THRESHOLD,
                                  use_color=False,
                                  expand_to_project=False):
    """
    Format custom coverage report with:
    - Relative file paths
    - Merged columns with 3-line headers
    - Branch coverage
    - Grouped by files, modules/libs, and total
    - Optional color coding for coverage thresholds
    """
    data = json_data['data'][0]
    files = data.get('files', [])

    # Aggregate data
    individual_files, groups, total = aggregate_coverage_data(
        files, project_root
    )

    # Optionally expand to the whole project by adding missing files
    if expand_to_project:
        covered_paths = {f["path"] for f in individual_files}
        for rel in enumerate_project_rs_files(project_root):
            if rel in covered_paths:
                continue
            category, group_name = categorize_file(rel)
            loc_total = count_non_empty_lines(Path(project_root) / rel)
            file_stats = {
                "path": rel,
                "regions": {"covered": 0, "total": 0},
                "functions": {"covered": 0, "total": 0},
                "lines": {"covered": 0, "total": loc_total},
            }
            individual_files.append(file_stats)
            if category in ["module", "lib"] and group_name:
                found = None
                for g in groups:
                    if g["name"] == group_name and g["type"] == category:
                        found = g
                        break
                if not found:
                    found = {
                        "name": group_name,
                        "type": category,
                        "regions": {"covered": 0, "total": 0},
                        "functions": {"covered": 0, "total": 0},
                        "lines": {"covered": 0, "total": 0},
                    }
                    groups.append(found)
                found["lines"]["total"] += loc_total
            total["lines"]["total"] += loc_total

    # Build report
    lines = [
        "=" * SEPARATOR_WIDTH,
        "COVERAGE REPORT",
        "=" * SEPARATOR_WIDTH,
        ""
    ]

    # Add summary info
    lines.append(
        f"Files covered: {len(individual_files)} out of "
        f"{len(files)} total instrumented files"
    )
    lines.append(f"Coverage threshold: {threshold}%")
    if use_color and supports_color():
        lines.append(
            "Color coding: \033[92mgreen\033[0m = above threshold, "
            "\033[91mred\033[0m = below threshold"
        )
    lines.append("")

    # Individual Files Section
    lines.extend(format_section_header("Individual Files:"))
    for file_stats in sorted(individual_files, key=lambda x: x['path']):
        lines.append(format_coverage_row(
            file_stats['path'], file_stats, threshold=threshold, use_color=use_color
        ))

    # Modules & Libs Section
    lines.append("")
    lines.extend(format_section_header("Modules & Libraries:"))
    for group in sorted(groups, key=lambda x: (x['type'], x['name'])):
        name = f"{group['type']}/{group['name']}"
        lines.append(format_coverage_row(
            name, group, threshold=threshold, use_color=use_color
        ))

    # Total Section
    lines.append("")
    lines.extend(format_section_header("Total:", separator="="))
    lines.append(format_coverage_row(
        "TOTAL", total, threshold=threshold, use_color=use_color
    ))
    lines.append("=" * SEPARATOR_WIDTH)

    return "\n".join(lines)


def collect_unit_coverage(
    output_dir,
    config_file=None,
    test_filter=None,
    skip_build=False
):
    """Collect coverage from unit tests.

    Args:
        output_dir: Directory to store coverage reports
        config_file: Optional config file path
        test_filter: Optional package filter (e.g., 'modkit-db')
        skip_build: If True, skip clean and test execution
    """
    if skip_build:
        print("Skipping test execution, using existing coverage data")
        return

    step("Collecting unit test coverage")

    # Clean previous coverage data
    run_cmd(["cargo", "llvm-cov", "clean", "--workspace"], cwd=PROJECT_ROOT)

    # Run tests with coverage (allow failures)
    env = os.environ.copy()

    # Set config file if provided
    if config_file:
        env["HYPERSPOT_CONFIG"] = str(config_file)
        print(f"Using config: {config_file}")

    # Build command
    cmd = ["cargo", "llvm-cov"]

    # Add package filter if provided, otherwise use workspace
    if test_filter:
        cmd.extend(["--package", test_filter])
        print(f"Filtering tests: package={test_filter}")
    else:
        cmd.append("--workspace")

    # Note: --branch flag requires nightly Rust and is unstable
    # Branch coverage will be 0 without it, but region coverage
    # provides good coverage metrics for Rust code
    cmd.extend(["--all-features", "--no-report"])

    result = run_cmd_allow_fail(cmd, env=env, cwd=PROJECT_ROOT)

    if result.returncode != 0:
        print(
            "WARNING: Some unit tests failed, "
            "but coverage was still collected"
        )
    else:
        print("OK. Unit test coverage collected")


def parse_bind_addr_port(config_file):
    """Parse the bind_addr from config file and extract port number.

    Args:
        config_file: Path to YAML config file

    Returns:
        int: Port number from api_ingress.bind_addr
    """
    config_path = PROJECT_ROOT / config_file
    with open(config_path, 'r') as f:
        config = yaml.safe_load(f)

    bind_addr = config.get('modules', {}).get('api_ingress', {}).get(
        'config', {}).get('bind_addr', '127.0.0.1:8080'
    )
    if ':' not in bind_addr:
        raise ValueError(f"Invalid bind_addr format: {bind_addr}")

    _, port_str = bind_addr.rsplit(':', 1)
    try:
        return int(port_str)
    except ValueError:
        raise ValueError(f"Invalid port number in bind_addr: {bind_addr}")


def check_port_available(port):
    """Check if a port is available for binding.

    Args:
        port: Port number to check

    Raises:
        SystemExit: If port is already in use
    """
    try:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.bind(("127.0.0.1", port))
    except OSError:
        print(
            f"ERROR: Port {port} is already in use. "
            "Please stop the process using it or choose a different port.",
            file=sys.stderr
        )
        sys.exit(1)


def start_instrumented_server(config_file, output_dir, port=None):
    """Start the hyperspot-server with coverage instrumentation.

    Args:
        config_file: Path to config file
        output_dir: Directory for server logs
        port: Optional port override (parsed from config if None)

    Returns:
        tuple: (server_process, log_file, actual_port)
    """
    if port is None:
        port = parse_bind_addr_port(config_file)

    # Check port availability
    check_port_available(port)

    # Create output directory and log file
    output_dir.mkdir(parents=True, exist_ok=True)
    log_file = output_dir / "hyperspot-server.log"

    step(
        f"Starting server with coverage instrumentation "
        f"(config: {config_file})"
    )
    print(f"Server logs will be written to: {log_file}")

    # Set up environment for coverage
    env2 = os.environ.copy()
    env2["LLVM_PROFILE_FILE"] = (
        "target/llvm-cov-target/hyperspot-%p-%m.profraw"
    )

    # Build server command
    cmd = [
        "cargo", "llvm-cov", "run",
        "--bin", "hyperspot-server",
        "--features", "users-info-example",
        "--no-report",
        "--",
        "--config", str(PROJECT_ROOT / config_file),
        "run"
    ]

    # Log the exact command for debugging
    print(
        f"[INFO] Running: LLVM_PROFILE_FILE={env2['LLVM_PROFILE_FILE']} "
        f"{' '.join(cmd)}"
    )

    # Start server
    server_process = subprocess.Popen(
        cmd,
        env=env2,
        cwd=PROJECT_ROOT,
        stdout=open(log_file, "w"),
        stderr=subprocess.STDOUT,
        start_new_session=True  # Create new process group for cleanup
    )

    return server_process, log_file, port


def run_e2e_tests(base_url, test_filter=None):
    """Run E2E tests against the server.

    Args:
        base_url: Base URL of the running server
        test_filter: Optional test filter

    Returns:
        subprocess.CompletedProcess: Result of pytest execution
    """
    step("Running E2E tests")
    test_env = os.environ.copy()
    test_env["E2E_BASE_URL"] = base_url

    # Check pytest is available
    result = run_cmd_allow_fail([PYTHON, "-m", "pytest", "--version"])
    if result.returncode != 0:
        print(
            "ERROR: pytest is not installed. Install with: "
            "pip install -r testing/e2e/requirements.txt",
            file=sys.stderr
        )
        sys.exit(1)

    # Build pytest command
    pytest_cmd = [PYTHON, "-m", "pytest", "testing/e2e", "-vv"]
    if test_filter:
        pytest_cmd.extend(["-k", test_filter])

    return run_cmd_allow_fail(
        pytest_cmd,
        env=test_env,
        cwd=PROJECT_ROOT
    )


def stop_server(server_process, port, log_file):
    """Stop the server process and verify cleanup.

    Args:
        server_process: Server subprocess object
        port: Port the server was running on
        log_file: Path to server log file
    """
    step("Stopping server")
    try:
        # Try graceful shutdown first (SIGINT to process group)
        os.killpg(os.getpgid(server_process.pid), signal.SIGINT)
    except Exception:
        try:
            server_process.terminate()
        except Exception:
            pass

    try:
        server_process.wait(timeout=15)
    except subprocess.TimeoutExpired:
        # Force kill entire process group
        try:
            os.killpg(os.getpgid(server_process.pid), signal.SIGKILL)
        except Exception:
            server_process.kill()
        server_process.wait()

    # Verify port is freed
    time.sleep(1)
    try:
        with socket.create_connection(("127.0.0.1", port), timeout=0.5):
            print(
                f"WARNING: Port {port} still occupied after shutdown. "
                "You may need to kill remaining processes manually.",
                file=sys.stderr
            )
    except OSError:
        pass  # Port is free, good

    print("[OK] Server stopped")
    print(f"[OK] Server logs saved to: {log_file}")


def collect_e2e_local_coverage(
    output_dir,
    config_file="config/e2e-local.yaml",
    test_filter=None,
    skip_build=False
):
    """Collect coverage from e2e tests running against local server.

    Args:
        output_dir: Directory to store coverage reports
        config_file: Config file for server
        test_filter: Optional test path filter (e.g., 'modules/api_ingress')
        skip_build: If True, skip clean and test execution
    """
    if skip_build:
        print("Skipping test execution, using existing coverage data")
        return

    step("Collecting local E2E test coverage")

    # Clean previous coverage data
    run_cmd(["cargo", "llvm-cov", "clean", "--workspace"], cwd=PROJECT_ROOT)

    # Start server with coverage instrumentation
    server_process, log_file, desired_port = start_instrumented_server(
        config_file, output_dir
    )
    base_url = f"http://127.0.0.1:{desired_port}"

    try:
        # Wait for server to be ready (TCP first, then HTTP health)
        wait_for_tcp("127.0.0.1", desired_port, timeout_secs=30, log_path=log_file)
        wait_for_health(base_url, timeout_secs=90, log_path=log_file)

        # Run e2e tests
        pytest_result = run_e2e_tests(base_url, test_filter)

        if pytest_result.returncode != 0:
            print(
                "WARNING: Some E2E tests failed, "
                "but coverage was still collected. "
                "Please check the test output for details."
            )
        else:
            print("[OK] E2E tests passed")

    finally:
        # Stop server and all child processes
        stop_server(server_process, desired_port, log_file)

        # Quick sanity check: did we produce any profile data?
        prof_dir = PROJECT_ROOT / "target" / "llvm-cov-target"
        prof_count = 0
        if prof_dir.exists():
            prof_count = sum(1 for p in prof_dir.glob("**/*.profraw"))
        print(
            f"[INFO] profile files found: {prof_count} in {prof_dir}"  # noqa: E501
        )
        # Give filesystem a moment to flush profile data on some platforms
        time.sleep(0.5)


def generate_reports(output_dir, mode, threshold=COVERAGE_THRESHOLD, use_color=False):
    """Generate coverage reports in multiple formats."""
    step(f"Generating coverage reports ({mode})")

    output_dir.mkdir(parents=True, exist_ok=True)

    # Decide scope: e2e -> only server package (its deps included); others -> workspace
    is_e2e_local = isinstance(mode, str) and mode.startswith("e2e-local")

    # Generate HTML report
    print("Generating HTML report...")
    html_dir = output_dir / "html"
    base = ["cargo", "llvm-cov", "report"]
    run_cmd(
        base
        + [
            "--html",
            "--output-dir",
            str(html_dir),
        ],
        cwd=PROJECT_ROOT,
    )

    # cargo-llvm-cov creates a nested html/ directory, move contents up
    nested_html = html_dir / "html"
    if nested_html.exists():
        import shutil
        for item in nested_html.iterdir():
            dest = html_dir / item.name
            # Remove destination if it exists
            if dest.exists():
                if dest.is_dir():
                    shutil.rmtree(dest)
                else:
                    dest.unlink()
            shutil.move(str(item), str(dest))
        nested_html.rmdir()

    print(f"[OK] HTML report: {html_dir / 'index.html'}")

    # Generate text report and capture it
    print("\nGenerating text report...")
    result = run_cmd_capture(
        base
        + [
            "--summary-only",
        ],
        cwd=PROJECT_ROOT,
    )

    text_report = result.stdout
    text_file = output_dir / "summary.txt"
    text_file.write_text(text_report)
    print(f"[OK] Text report: {text_file}")

    # Generate LCOV report
    print("\nGenerating LCOV report...")
    lcov_file = output_dir / "lcov.info"
    run_cmd(
        base
        + [
            "--lcov",
            "--output-path",
            str(lcov_file),
        ],
        cwd=PROJECT_ROOT,
    )
    print(f"[OK] LCOV report: {lcov_file}")

    # Generate JSON report
    print("\nGenerating JSON report...")
    json_result = run_cmd_capture(
        base
        + [
            "--json",
        ],
        cwd=PROJECT_ROOT,
    )

    json_file = output_dir / "coverage.json"
    json_file.write_text(json_result.stdout)
    print(f"[OK] JSON report: {json_file}")

    # Generate custom formatted report
    print("\nGenerating custom coverage report...")
    json_data = json.loads(json_result.stdout)
    custom_report = format_custom_coverage_report(
        json_data,
        PROJECT_ROOT,
        threshold,
        use_color,
        expand_to_project=is_e2e_local,
    )

    # Save custom report (without color codes)
    custom_file = output_dir / "coverage_report.txt"
    custom_file.write_text(
        format_custom_coverage_report(
            json_data,
            PROJECT_ROOT,
            threshold,
            use_color=False,
            expand_to_project=is_e2e_local,
        )
    )
    print(f"[OK] Custom report: {custom_file}")

    # Print custom report to stdout (with color if supported)
    print("\n" + custom_report)

    return custom_report


def run_coverage_workflow(mode, output_dir, config_file, test_filter, skip_build, threshold):
    """Common workflow for running coverage collection and report generation.

    Args:
        mode: 'unit' or 'e2e-local'
        output_dir: Directory for coverage output
        config_file: Config file path (or None for unit)
        test_filter: Optional test filter
        skip_build: Whether to skip build/test execution
        threshold: Coverage threshold percentage
    """
    use_color = supports_color()  # Auto-detect color support

    if mode == "unit":
        collect_unit_coverage(output_dir, config_file, test_filter, skip_build)
        report_mode = "unit tests"
    elif mode == "e2e-local":
        collect_e2e_local_coverage(output_dir, config_file, test_filter, skip_build)
        report_mode = "e2e-local tests"
    else:
        raise ValueError(f"Unknown mode: {mode}")

    generate_reports(output_dir, report_mode, threshold, use_color)

    print(f"\n[OK] {report_mode.capitalize()} coverage reports generated in: {output_dir}")


def cmd_coverage_unit(args):
    """Generate coverage for unit tests only."""
    output_dir = COVERAGE_DIR / "unit"
    config_file = PROJECT_ROOT / args.config if args.config else None
    test_filter = args.filter if hasattr(args, 'filter') else None
    skip_build = args.skip_build if hasattr(args, 'skip_build') else False
    threshold = args.threshold if hasattr(args, 'threshold') else COVERAGE_THRESHOLD

    run_coverage_workflow("unit", output_dir, config_file, test_filter, skip_build, threshold)


def cmd_coverage_e2e(args):
    """Generate coverage for e2e tests only."""
    output_dir = COVERAGE_DIR / "e2e-local"
    config_file = args.config if args.config else "config/e2e-local.yaml"
    test_filter = args.filter if hasattr(args, 'filter') else None
    skip_build = args.no_build if hasattr(args, 'no_build') else False
    threshold = args.threshold if hasattr(args, 'threshold') else COVERAGE_THRESHOLD

    run_coverage_workflow("e2e-local", output_dir, config_file, test_filter, skip_build, threshold)


def cmd_coverage_combined(args):
    """Generate combined coverage from both unit and e2e tests."""
    output_dir = COVERAGE_DIR / "combined"
    config_file = args.config if args.config else "config/e2e-local.yaml"
    threshold = args.threshold if hasattr(args, 'threshold') else COVERAGE_THRESHOLD
    use_color = supports_color()  # Auto-detect color support

    # Clean previous coverage data
    step("Cleaning previous coverage data")
    run_cmd(["cargo", "llvm-cov", "clean", "--workspace"], cwd=PROJECT_ROOT)

    # Collect unit test coverage
    step("Collecting unit test coverage")
    env = os.environ.copy()

    # Set config file for unit tests
    env["HYPERSPOT_CONFIG"] = config_file
    print(f"Using config: {config_file}")

    result = run_cmd_allow_fail([
        "cargo", "llvm-cov",
        "--workspace",
        "--all-features",
        "--no-report"
    ], env=env, cwd=PROJECT_ROOT)

    if result.returncode != 0:
        print(
            "WARNING: Some unit tests failed, "
            "but coverage was still collected"
        )
    else:
        print("[OK] Unit test coverage collected")

    # Collect e2e coverage (without cleaning)
    step("Collecting E2E test coverage for combined mode")
    server_process, log_file, port = start_instrumented_server(
        config_file, output_dir
    )
    base_url = f"http://127.0.0.1:{port}"

    try:
        # Wait for server to be ready
        wait_for_tcp("127.0.0.1", port, timeout_secs=30, log_path=log_file)
        wait_for_health(base_url, timeout_secs=60, log_path=log_file)

        # Run E2E tests
        pytest_result = run_e2e_tests(base_url)

        if pytest_result.returncode != 0:
            print(
                "WARNING: Some E2E tests failed, "
                "but coverage was still collected"
            )
        else:
            print("[OK] E2E tests passed")

    finally:
        stop_server(server_process, port, log_file)

    # Generate combined reports
    generate_reports(output_dir, "combined (unit + e2e)", threshold, use_color)

    print(
        f"\n[OK] Combined coverage reports generated in: {output_dir}"
    )


def validate_environment(command):
    """
    Validate that the environment has the necessary
    prerequisites for the given command.

    Args:
        command: The coverage command being run
        ('unit', 'e2e-local', 'combined')

    Raises:
        SystemExit: If environment validation fails
    """
    step("Validating test environment")

    if command == "unit":
        env_type = "core"
    elif command in ["e2e-local", "combined"]:
        env_type = "e2e"
    else:
        env_type = "core"

    print(f"Checking {env_type} prerequisites for {command} coverage...")

    if not check_environment_ready(env_type):
        print("\nERROR: Environment validation failed for "
              "{} coverage.".format(command))
        print("Please install missing prerequisites and try again.")
        print("You can run 'python scripts/check_test_env.py --mode core' "
              "or 'python scripts/check_test_env.py --mode e2e' "
              "to see detailed requirements.")
        sys.exit(1)

    print("Environment validation passed!")


def main():
    """Main entry point."""
    # Ensure we're in the project root
    os.chdir(PROJECT_ROOT)

    # Check for cargo-llvm-cov
    ensure_tool("cargo-llvm-cov", "cargo install cargo-llvm-cov")

    parser = argparse.ArgumentParser(
        description="Generate code coverage reports for HyperSpot",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Generate unit test coverage only
  python scripts/coverage.py unit

  # Generate unit test coverage for specific package
  python scripts/coverage.py unit --filter modkit-db

  # Generate local e2e test coverage only
  python scripts/coverage.py e2e-local

  # Generate local e2e test coverage for specific module
  python scripts/coverage.py e2e-local --filter modules/api_ingress

  # Generate combined coverage (unit + e2e-local)
  python scripts/coverage.py combined
"""
    )

    subparsers = parser.add_subparsers(dest="command", required=True)

    # Unit coverage
    p_unit = subparsers.add_parser(
        "unit",
        help="Generate coverage from unit tests only"
    )
    p_unit.add_argument(
        "--config",
        help=(
            "Config file to use (relative to project root, "
            "e.g., config/e2e-local.yaml)"
        ),
        default=None
    )
    p_unit.add_argument(
        "--filter",
        help="Filter tests by package name (e.g., modkit-db, api-ingress)",
        default=None
    )
    p_unit.add_argument(
        "--skip-build",
        action="store_true",
        help="Skip test execution, only generate reports from existing data"
    )
    p_unit.add_argument(
        "--threshold",
        type=int,
        default=COVERAGE_THRESHOLD,
        help="Coverage threshold percentage for warnings (default: %s)" %
             COVERAGE_THRESHOLD
    )
    p_unit.add_argument(
        "--skip-env-check",
        action="store_true",
        help="Skip environment prerequisite validation (not recommended)"
    )
    p_unit.set_defaults(func=cmd_coverage_unit)

    # E2E coverage
    p_e2e = subparsers.add_parser(
        "e2e-local",
        help="Generate coverage from e2e tests only"
    )
    p_e2e.add_argument(
        "--config",
        help="Config file to use (relative to project root)",
        default="config/e2e-local.yaml"
    )
    p_e2e.add_argument(
        "--filter",
        help=(
            "Filter E2E tests by path relative to testing/e2e "
            "(e.g., modules/api_ingress)"
        ),
        default=None
    )
    p_e2e.add_argument(
        "--skip-build",
        action="store_true",
        help="Skip test execution, only generate reports from existing data"
    )
    p_e2e.add_argument(
        "--threshold",
        type=int,
        default=COVERAGE_THRESHOLD,
        help="Coverage threshold percentage for warnings (default: %s)" %
             COVERAGE_THRESHOLD
    )
    p_e2e.add_argument(
        "--skip-env-check",
        action="store_true",
        help="Skip environment prerequisite validation (not recommended)"
    )
    p_e2e.set_defaults(func=cmd_coverage_e2e)

    # Combined coverage
    p_combined = subparsers.add_parser(
        "combined",
        help="Generate combined coverage (unit + e2e)"
    )
    p_combined.add_argument(
        "--config",
        help="Config file to use (relative to project root)",
        default="config/e2e-local.yaml"
    )
    p_combined.add_argument(
        "--threshold",
        type=int,
        default=COVERAGE_THRESHOLD,
        help="Coverage threshold percentage for warnings (default: %s)" %
             COVERAGE_THRESHOLD
    )
    p_combined.add_argument(
        "--skip-env-check",
        action="store_true",
        help="Skip environment prerequisite validation (not recommended)"
    )
    p_combined.set_defaults(func=cmd_coverage_combined)

    args = parser.parse_args()

    # Validate environment prerequisites before proceeding (unless skipped)
    if not hasattr(args, 'skip_env_check') or not args.skip_env_check:
        validate_environment(args.command)
    else:
        print("WARNING: Skipping environment prerequisite validation")
        print("This may cause failures if required tools are not installed.")

    args.func(args)


if __name__ == "__main__":
    main()
