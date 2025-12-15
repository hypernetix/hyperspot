#!/usr/bin/env python3
"""
Test dylint lints on UI test cases.
Compiles the UI test crate and verifies expected violations are detected.
"""

import subprocess
import sys
import os
import re
from pathlib import Path
from collections import defaultdict


def supports_color():
    """Check if the terminal supports ANSI colors."""
    # No color if output is not a TTY (e.g., piped to file)
    if not hasattr(sys.stdout, 'isatty') or not sys.stdout.isatty():
        return False
    # Check for explicit NO_COLOR environment variable
    if os.environ.get('NO_COLOR'):
        return False
    # Check TERM environment variable
    term = os.environ.get('TERM', '')
    if term == 'dumb':
        return False
    # Windows needs special handling
    if sys.platform == 'win32':
        # Windows 10+ supports ANSI if TERM is set or if running in modern terminal
        return 'ANSICON' in os.environ or term or os.environ.get('WT_SESSION')
    return True


# ANSI colors (only if terminal supports it)
if supports_color():
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    RESET = '\033[0m'
    BOLD = '\033[1m'
    DIM = '\033[2m'
else:
    GREEN = RED = YELLOW = BLUE = RESET = BOLD = DIM = ''


def get_toolchain_info():
    """Get host toolchain and rustup toolchain from rust-toolchain.toml."""
    result = subprocess.run(
        ['rustc', '--version', '--verbose'],
        capture_output=True, text=True
    )
    host = None
    for line in result.stdout.split('\n'):
        if line.startswith('host:'):
            host = line.split(':')[1].strip()
            break

    script_dir = Path(__file__).parent
    toolchain_file = script_dir / 'rust-toolchain.toml'
    rustup_toolchain = 'nightly'
    if toolchain_file.exists():
        with open(toolchain_file) as f:
            for line in f:
                if 'channel' in line:
                    match = re.search(r'"([^"]+)"', line)
                    if match:
                        rustup_toolchain = match.group(1)
                    break

    return host, rustup_toolchain


def find_dylint_lib(script_dir):
    """Find the dylint library in target/release."""
    target_dir = script_dir / 'target' / 'release'

    for ext in ['.dylib', '.so', '.dll']:
        pattern = f'libcontract_lints@*{ext}' if ext != '.dll' else f'contract_lints@*{ext}'
        libs = list(target_dir.glob(pattern))
        if libs:
            return libs[0].resolve()

    return None


def build_dylint_lints(script_dir, rustup_toolchain, host):
    """Build the dylint lints and ensure proper naming."""
    print("Building dylint lints...")

    result = subprocess.run(
        ['cargo', 'build', '--release', '--quiet'],
        cwd=script_dir,
        capture_output=True, text=True
    )

    if result.returncode != 0:
        print(f"{RED}Build failed:{RESET}")
        print(result.stderr)
        return None

    target_dir = script_dir / 'target' / 'release'
    lib_name = f"libcontract_lints@{rustup_toolchain}-{host}"

    for src_ext, dst_ext in [('.dylib', '.dylib'), ('.so', '.so'), ('.dll', '.dll')]:
        src = target_dir / f"libcontract_lints{src_ext}"
        if src_ext == '.dll':
            src = target_dir / f"contract_lints{src_ext}"
        if src.exists():
            dst = target_dir / f"{lib_name}{dst_ext}"
            if src_ext == '.dll':
                dst = target_dir / f"contract_lints@{rustup_toolchain}-{host}{dst_ext}"
            try:
                import shutil
                shutil.copy2(src, dst)
            except:
                pass

    return find_dylint_lib(script_dir)


def get_driver_path(rustup_toolchain, host):
    """Get path to dylint driver, install if missing."""
    driver_path = Path.home() / '.dylint_drivers' / f'{rustup_toolchain}-{host}' / 'dylint-driver'

    if not driver_path.exists():
        print(f"Installing dylint driver for {rustup_toolchain}...")
        subprocess.run(
            ['cargo', f'+{rustup_toolchain}', 'install', 'cargo-dylint', 'dylint-link', '--quiet'],
            capture_output=True
        )

    return driver_path


def discover_test_modules(ui_dir):
    """Discover all test modules in src/ directory.

    Scans for files matching pattern de{XXXX}_test.rs and extracts
    lint ID and description from the first line comment.

    Returns list of tuples: (relative_path, lint_id, description)
    """
    test_modules = []
    src_dir = ui_dir / 'src'

    for test_file in src_dir.rglob('de*_test.rs'):
        # Extract lint ID from filename (e.g., de0101_test.rs -> DE0101)
        filename = test_file.stem  # de0101_test
        match = re.match(r'de(\d{4})_test', filename)
        if not match:
            continue

        lint_id = f"DE{match.group(1)}"

        # Try to extract description from first line comment
        description = lint_id  # Default to lint ID if no description found
        try:
            with open(test_file, 'r') as f:
                first_line = f.readline().strip()
                # Pattern: // Test DE0101: Description Here
                desc_match = re.search(rf'//\s*Test\s+{lint_id}:\s*(.+)', first_line)
                if desc_match:
                    description = desc_match.group(1).strip()
        except Exception:
            pass

        # Get path relative to ui_dir
        relative_path = str(test_file.relative_to(ui_dir))
        test_modules.append((relative_path, lint_id, description))

    # Sort by lint ID for consistent ordering
    test_modules.sort(key=lambda x: x[1])
    return test_modules


def get_test_expectations(filepath, lint_id):
    """Get line numbers where violations are expected/not expected for a specific lint."""
    should_trigger = []  # Lines where violation IS expected
    should_not_trigger = []  # Lines where violation is NOT expected

    with open(filepath, 'r') as f:
        lines = f.readlines()
        for i, line in enumerate(lines):
            # Check if this comment specifies the lint ID or no specific lint
            matches_lint = lint_id in line or 'DE0' not in line

            if '// Should trigger' in line and 'NOT' not in line and matches_lint:
                # Find the code line this comment refers to
                for j in range(i + 1, min(i + 5, len(lines))):
                    next_line = lines[j].strip()
                    if next_line and not next_line.startswith('//'):
                        should_trigger.append(j + 1)  # 1-indexed
                        break

            elif '// Should NOT trigger' in line and matches_lint:
                # Find the code line this comment refers to
                for j in range(i + 1, min(i + 5, len(lines))):
                    next_line = lines[j].strip()
                    if next_line and not next_line.startswith('//'):
                        should_not_trigger.append(j + 1)  # 1-indexed
                        break

    return should_trigger, should_not_trigger


def compile_with_dylint(ui_dir, dylint_lib, driver_path, rustup_toolchain):
    """Compile entire UI test library with dylint and capture output."""
    env = os.environ.copy()
    env['RUSTC_WORKSPACE_WRAPPER'] = str(driver_path)
    env['DYLINT_LIBS'] = f'["{dylint_lib}"]'
    env['DYLINT_METADATA'] = 'null'
    env['DYLINT_NO_DEPS'] = '0'
    env['RUSTUP_TOOLCHAIN'] = rustup_toolchain

    cargo_toml = ui_dir / 'Cargo.toml'

    if not cargo_toml.exists():
        return 1, f"error: Cargo.toml not found at {cargo_toml}\n"

    cmd = [
        'cargo', f'+{rustup_toolchain}', 'check',
        '--manifest-path', str(cargo_toml),
        '--lib', '--message-format', 'short'
    ]

    result = subprocess.run(cmd, env=env, capture_output=True, text=True)
    return result.returncode, result.stderr + result.stdout


def extract_violations(output):
    """Extract dylint violations from compiler output."""
    violations = defaultdict(list)

    for line in output.split('\n'):
        if 'DE' in line and ('error' in line or 'warning' in line):
            match = re.search(r'DE(\d{4})', line)
            if match:
                lint_id = f"DE{match.group(1)}"
                file_match = re.match(r'^([^:]+):(\d+):(\d+):', line)
                violations[lint_id].append({
                    'raw': line.strip(),
                    'file': file_match.group(1) if file_match else '',
                    'line': int(file_match.group(2)) if file_match else 0
                })

    return violations


def main():
    script_dir = Path(__file__).parent.resolve()
    ui_dir = script_dir / 'contract_lints' / 'ui'

    host, rustup_toolchain = get_toolchain_info()
    if not host:
        print(f"{RED}Failed to determine host toolchain{RESET}")
        sys.exit(1)

    dylint_lib = build_dylint_lints(script_dir, rustup_toolchain, host)
    if not dylint_lib:
        print(f"{RED}Failed to find dylint library after build{RESET}")
        sys.exit(1)

    driver_path = get_driver_path(rustup_toolchain, host)
    if not driver_path.exists():
        print(f"{RED}Dylint driver not found at {driver_path}{RESET}")
        sys.exit(1)

    print(f"\n{BOLD}Testing Dylint Lints on UI Test Crate{RESET}")
    print("=" * 70)

    print(f"\nCompiling with dylint ({rustup_toolchain})...")
    returncode, output = compile_with_dylint(ui_dir, dylint_lib, driver_path, rustup_toolchain)

    all_violations = extract_violations(output)

    # Discover test modules dynamically from src/
    test_modules = discover_test_modules(ui_dir)
    print(f"\nDiscovered {len(test_modules)} test module(s)")

    total_passed = 0
    total_failed = 0
    total_expected = 0
    total_triggered = 0

    for module_path, lint_id, description in test_modules:
        full_path = ui_dir / module_path
        if not full_path.exists():
            continue

        # Count violations for this lint from this specific file
        actual_list = [v for v in all_violations.get(lint_id, []) if module_path in v['file']]
        actual_lines = [v['line'] for v in actual_list]

        # Get expected/not-expected violation lines
        should_trigger, should_not_trigger = get_test_expectations(full_path, lint_id)

        print(f"\n{BLUE}→ {module_path}{RESET} ({lint_id}: {description})")
        print("  " + "─" * 66)

        # Calculate errors
        errors = 0
        results = []

        # Check "Should trigger" cases
        for line in should_trigger:
            total_expected += 1
            triggered = any(abs(line - al) <= 2 for al in actual_lines)
            if triggered:
                total_triggered += 1
                results.append((line, True, True, f"triggered {lint_id} as expected"))
            else:
                results.append((line, True, False, f"NOT triggered {lint_id} (should have!)"))
                errors += 1

        # Check "Should NOT trigger" cases
        for line in should_not_trigger:
            triggered = any(abs(line - al) <= 2 for al in actual_lines)
            if not triggered:
                results.append((line, False, False, f"not triggered {lint_id} as expected"))
            else:
                results.append((line, False, True, f"TRIGGERED {lint_id} (should not!)"))
                errors += 1

        # Check for unexpected violations (not covered by any comment)
        for v in actual_list:
            line = v['line']
            covered = any(abs(line - el) <= 2 for el in should_trigger + should_not_trigger)
            if not covered:
                results.append((line, None, True, f"unexpected {lint_id} violation"))
                errors += 1

        passed = errors == 0
        status = f"{GREEN}✓ PASS{RESET}" if passed else f"{RED}✗ FAIL ({errors} errors){RESET}"
        print(f"  {status}")

        # Sort results by line number and show
        results.sort(key=lambda x: x[0])
        for line, should, did, msg in results:
            filename = Path(full_path).name
            if should is True and did:  # Should trigger and did
                print(f"    {GREEN}✓{RESET} {filename}:{line}: {msg}")
            elif should is False and not did:  # Should NOT trigger and didn't
                print(f"    {GREEN}✓{RESET} {filename}:{line}: {msg}")
            else:  # Error case
                print(f"    {RED}✗{RESET} {filename}:{line}: {msg}")

        if passed:
            total_passed += 1
        else:
            total_failed += 1

    # Summary of all violations by lint
    print(f"\n{'='*70}")
    print(f"\n{BOLD}All Violations by Lint:{RESET}")

    total_violations = 0
    for lint_id in sorted(all_violations.keys()):
        violations = all_violations[lint_id]
        total_violations += len(violations)
        print(f"\n  {BOLD}{lint_id}{RESET} ({len(violations)} violations):")
        for v in violations:
            filename = Path(v['file']).name
            msg = v['raw'].split(': ', 2)[-1] if ': ' in v['raw'] else v['raw']
            print(f"    {filename}:{v['line']}: {msg}")

    # Summary
    print(f"\n{'='*70}")
    print(f"\n{BOLD}Summary:{RESET}")
    print(f"  Tests: {GREEN}{total_passed} passed{RESET}, {RED if total_failed else DIM}{total_failed} failed{RESET}")

    # Final report
    if total_expected > 0:
        pct = (total_triggered / total_expected) * 100
        if total_triggered == total_expected:
            print(f"  Total violations detected: {total_triggered} out of {total_expected} ({pct:.0f}%). {GREEN}OK{RESET}")
        else:
            print(f"  Total violations detected: {total_triggered} out of {total_expected} ({pct:.0f}%). {RED}FAIL{RESET}")

    if total_failed > 0:
        print(f"\n{RED}✗ Some tests failed{RESET}")
        sys.exit(1)
    else:
        print(f"\n{GREEN}✓ All tests passed!{RESET}")
        sys.exit(0)


if __name__ == '__main__':
    main()
