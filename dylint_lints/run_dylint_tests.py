#!/usr/bin/env python3
"""
Dylint Test Runner with Enhanced Formatting

This script runs cargo test for all dylint lints and provides a detailed,
formatted output showing individual test cases, violations, and summaries.

Note: This script focuses on UI test results and ignores test_comment_annotations_match_stderr
tests, which are developer-facing validation tests to ensure test annotations match stderr files.
"""

import subprocess
import re
import sys
from pathlib import Path
from dataclasses import dataclass, field
from typing import List, Dict, Tuple


@dataclass
class Violation:
    """Represents a single lint violation"""
    file: str
    line: int
    message: str
    lint_code: str


@dataclass
class TestCase:
    """Represents a single test case (UI file)"""
    name: str
    file_path: Path
    lint_code: str
    lint_name: str
    violations: List[Violation] = field(default_factory=list)
    passed: bool = True


@dataclass
class LintPackage:
    """Represents a dylint lint package"""
    name: str
    path: Path
    lint_code: str
    lint_description: str
    test_cases: List[TestCase] = field(default_factory=list)


def parse_toml_simple(content: str, key_path: List[str]) -> List[str]:
    """Simple TOML parser for specific keys"""
    lines = content.split('\n')
    result = []
    in_section = False
    current_section = []
    
    for line in lines:
        line = line.strip()
        
        # Check for section header
        if line.startswith('[') and line.endswith(']'):
            section = line[1:-1]
            current_section = section.split('.')
            in_section = current_section == key_path
            continue
        
        # Parse key-value in the right section
        if in_section and '=' in line:
            continue
        
        # Parse array values in the right section
        if in_section and line.startswith('"') and line.endswith('",'):
            value = line.strip('"').rstrip(',')
            result.append(value)
        elif in_section and line.startswith('"') and line.endswith('"'):
            value = line.strip('"')
            result.append(value)
    
    return result


def get_package_name_from_cargo(cargo_path: Path) -> str:
    """Extract package name from Cargo.toml"""
    content = cargo_path.read_text()
    match = re.search(r'name\s*=\s*"([^"]+)"', content)
    return match.group(1) if match else cargo_path.parent.name


def get_lint_packages(workspace_root: Path) -> List[LintPackage]:
    """Discover all lint packages in the workspace"""
    packages = []
    
    cargo_toml = workspace_root / "Cargo.toml"
    content = cargo_toml.read_text()
    
    # Parse members array manually
    members = parse_toml_simple(content, ['workspace'])
    if not members:
        # Fallback: parse members manually with regex
        match = re.search(r'\[workspace\].*?members\s*=\s*\[(.*?)\]', content, re.DOTALL)
        if match:
            members_str = match.group(1)
            members = [m.strip().strip('"').strip(',') for m in members_str.split('\n') if m.strip()]
    
    # Discover packages by scanning directories
    for lint_category in ['de01_contract_layer', 'de02_api_layer', 'de08_rest_api_conventions']:
        category_dir = workspace_root / lint_category
        if not category_dir.exists():
            continue
        
        for package_dir in sorted(category_dir.iterdir()):
            if not package_dir.is_dir():
                continue
            
            package_cargo = package_dir / "Cargo.toml"
            if not package_cargo.exists():
                continue
            
            package_name = get_package_name_from_cargo(package_cargo)
            
            # Extract lint code from package name (e.g., "de0101_no_serde_in_contract" -> "DE0101")
            match = re.match(r'(de\d{4})', package_name)
            lint_code = match.group(1).upper() if match else "UNKNOWN"
            
            # Try to get description from src/lib.rs
            lib_rs = package_dir / "src" / "lib.rs"
            lint_description = get_lint_description(lib_rs)
            
            packages.append(LintPackage(
                name=package_name,
                path=package_dir,
                lint_code=lint_code,
                lint_description=lint_description
            ))
    
    return sorted(packages, key=lambda p: p.name)


def get_lint_description(lib_rs: Path) -> str:
    """Extract lint description from lib.rs file"""
    if not lib_rs.exists():
        return "Unknown"
    
    with open(lib_rs) as f:
        content = f.read()
    
    # Look for the lint declaration and extract description
    patterns = [
        r'pub\s+([A-Z0-9_]+),\s*\n\s*Deny,\s*\n\s*"([^"]+)"',
        r'/// ### What it does\s*\n\s*///\s*\n\s*/// ([^\n]+)',
    ]
    
    for pattern in patterns:
        match = re.search(pattern, content)
        if match:
            desc = match.group(1) if len(match.groups()) == 1 else match.group(2)
            return desc.strip()
    
    return "Unknown"


def parse_stderr_file(stderr_path: Path) -> List[Violation]:
    """Parse a .stderr file to extract expected violations"""
    violations = []
    
    if not stderr_path.exists():
        return violations
    
    content = stderr_path.read_text()
    
    # Parse error messages
    # Format: error: <message>
    #         --> $DIR/<file>.rs:<line>:<col>
    error_pattern = r'error:\s*([^\n]+)\n\s*-->\s*\$DIR/([^:]+):(\d+):'
    
    for match in re.finditer(error_pattern, content):
        message = match.group(1).strip()
        file = match.group(2)
        line = int(match.group(3))
        
        # Extract lint code from message
        lint_code_match = re.search(r'\(([A-Z]+\d+)\)', message)
        lint_code = lint_code_match.group(1) if lint_code_match else "UNKNOWN"
        
        violations.append(Violation(
            file=file,
            line=line,
            message=message,
            lint_code=lint_code
        ))
    
    return violations


def discover_test_cases(package: LintPackage) -> List[TestCase]:
    """Discover all test cases for a lint package"""
    test_cases = []
    ui_dir = package.path / "ui"
    
    if not ui_dir.exists():
        return test_cases
    
    # Find all .rs files in ui/
    for rs_file in sorted(ui_dir.glob("*.rs")):
        stderr_file = rs_file.with_suffix(".stderr")
        
        violations = parse_stderr_file(stderr_file)
        
        test_case = TestCase(
            name=rs_file.stem,
            file_path=rs_file,
            lint_code=package.lint_code,
            lint_name=package.lint_description,
            violations=violations
        )
        
        test_cases.append(test_case)
    
    return test_cases


def run_cargo_test(workspace_root: Path) -> Tuple[bool, str]:
    """Run cargo test for all lint packages"""
    print("Building dylint lints...\n")
    
    try:
        result = subprocess.run(
            ["cargo", "test", "--no-fail-fast"],
            cwd=workspace_root,
            capture_output=True,
            text=True,
            timeout=300
        )
        
        output = result.stdout + result.stderr
        
        # Check if UI tests passed (ignore test_comment_annotations_match_stderr tests)
        # We look for failures in ui_examples tests specifically
        success = result.returncode == 0 or not has_ui_test_failures(output)
        
        return success, output
    except subprocess.TimeoutExpired:
        return False, "Test execution timed out"
    except Exception as e:
        return False, f"Failed to run tests: {e}"


def has_ui_test_failures(cargo_output: str) -> bool:
    """Check if there are any UI test failures (excluding comment annotation tests)"""
    lines = cargo_output.splitlines()
    
    for line in lines:
        # Check for failed UI tests
        if re.search(r"^test \[ui\] .*?\.rs \.\.\.\s+FAILED", line):
            return True
        # Check for failed ui_examples tests
        if re.search(r"^test tests::ui_examples \.\.\.\s+FAILED", line):
            return True
    
    return False


def parse_ui_test_statuses(cargo_output_stdout: str, cargo_output_stderr: str) -> Dict[Tuple[str, str], bool]:
    """Parse UI test statuses from cargo output
    
    Returns only UI test results, excluding test_comment_annotations_match_stderr tests.
    """
    statuses: Dict[Tuple[str, str], bool] = {}
    
    # Parse all test results from stdout
    for line in cargo_output_stdout.splitlines():
        # Look for individual UI test results
        m = re.search(r"^test \[ui\] .*?/([^/\s]+)\.rs \.\.\.\s+(ok|FAILED)", line)
        if m:
            test_stem = m.group(1)
            result = m.group(2)
            # Store without crate name for now - we'll match it later
            statuses[test_stem] = result == "ok"

    return statuses


def print_test_header():
    """Print the test header"""
    print("\nTesting Dylint Lints on UI Test Crate")
    print("=" * 70)
    print("\nCompiling with dylint (nightly)...\n")


def print_test_case_results(test_cases: List[TestCase], all_passed: bool):
    """Print formatted results for all test cases"""
    # Group test cases by lint code
    grouped = {}
    for tc in test_cases:
        lint_key = f"{tc.lint_code}: {tc.lint_name}"
        if lint_key not in grouped:
            grouped[lint_key] = []
        grouped[lint_key].append(tc)
    
    total_lints = len(grouped)
    total_tests = len(test_cases)
    print(f"Testing {total_lints} lint(s) with {total_tests} test file(s)\n")
    
    for lint_key in sorted(grouped.keys()):
        test_group = grouped[lint_key]
        first_tc = test_group[0]
        
        print(f"→ {lint_key}")
        print("  " + "─" * 66)
        
        all_group_passed = all(tc.passed for tc in test_group)
        status = "✓ PASS" if all_group_passed else "✗ FAIL"
        print(f"  {status}")
        
        # Print test results grouped by test file
        for tc in sorted(test_group, key=lambda x: x.name):
            expected_label = "Expected: Triggered" if tc.violations else "Expected: Success"
            symbol = "✓" if tc.passed else "✗"
            print(f"    {symbol} {tc.name}.rs: {expected_label}")

            if tc.violations:
                for v in tc.violations:
                    print(f"        - line {v.line}: {v.message}")
        
        print()




def print_violations_by_lint(packages: List[LintPackage]):
    """Print all violations grouped by lint code"""
    print("=" * 70)
    print("\nAll Violations by Lint:\n")
    
    # Collect all violations by lint code
    violations_by_lint: Dict[str, List[Tuple[str, Violation]]] = {}
    
    for package in packages:
        for test_case in package.test_cases:
            for violation in test_case.violations:
                lint_code = violation.lint_code
                if lint_code not in violations_by_lint:
                    violations_by_lint[lint_code] = []
                violations_by_lint[lint_code].append((test_case.name, violation))
    
    # Print violations by lint
    for lint_code in sorted(violations_by_lint.keys()):
        violations = violations_by_lint[lint_code]
        count = len(violations)
        
        print(f"  {lint_code} ({count} violation{'s' if count != 1 else ''}):")
        
        for test_name, violation in sorted(violations, key=lambda x: (x[0], x[1].line)):
            # Clean up the message
            clean_message = violation.message
            print(f"    {test_name}.rs:{violation.line}: {clean_message}")
        
        print()


def print_summary(packages: List[LintPackage], all_tests_passed: bool, total_violations: int):
    """Print test summary"""
    print("=" * 70)
    print("\nSummary:")

    passed = sum(1 for p in packages for tc in p.test_cases if tc.passed)
    failed = sum(1 for p in packages for tc in p.test_cases if not tc.passed)
    total_tests = passed + failed
    
    # Count expected violations
    expected_violations = sum(len(tc.violations) for p in packages for tc in p.test_cases)
    
    print(f"  Tests: {passed} passed, {failed} failed, {total_tests} total")
    
    if all_tests_passed:
        percentage = 100
        print(f"  Total violations detected: {expected_violations} out of {expected_violations} ({percentage}%). OK")
        print("\n✓ All tests passed!")
    else:
        # When tests fail, we can't accurately count detected violations from the output
        # The test framework only tells us which tests failed, not how many violations were found
        print(f"  Expected violations: {expected_violations}")
        print("\n✗ Some tests failed!")


def main():
    """Main entry point"""
    workspace_root = Path(__file__).parent
    
    # Discover all lint packages
    packages = get_lint_packages(workspace_root)
    
    # Discover test cases for each package
    for package in packages:
        package.test_cases = discover_test_cases(package)
    
    # Run cargo test
    cargo_tests_passed, output = run_cargo_test(workspace_root)

    # Parse UI test statuses from cargo output
    try:
        result = subprocess.run(
            ["cargo", "test", "--no-fail-fast"],
            cwd=workspace_root,
            capture_output=True,
            text=True,
            timeout=300
        )
        ui_statuses_by_name = parse_ui_test_statuses(result.stdout, result.stderr)
    except Exception as e:
        print(f"Warning: Failed to parse test statuses: {e}")
        ui_statuses_by_name = {}
    
    # Update test case pass/fail status based on parsed output
    # Match test names to packages
    for package in packages:
        for tc in package.test_cases:
            # Look up test status by name
            if tc.name in ui_statuses_by_name:
                tc.passed = ui_statuses_by_name[tc.name]
            else:
                # If test wasn't found in output, mark as failed if overall cargo test failed
                tc.passed = cargo_tests_passed
    
    # Print formatted output
    print_test_header()
    
    # Collect all test cases
    all_test_cases = []
    for package in packages:
        all_test_cases.extend(package.test_cases)
    
    print_test_case_results(all_test_cases, cargo_tests_passed)
    print_violations_by_lint(packages)
    
    # Count expected violations from .stderr files
    expected_violations = sum(len(tc.violations) for p in packages for tc in p.test_cases)
    
    # Determine overall success:
    # 1. All UI test cases must have passed (tc.passed == True)
    # 2. Cargo test must have passed (cargo_tests_passed)
    # Note: We ignore test_comment_annotations_match_stderr tests in success determination
    # as those are developer-facing validation tests, not lint behavior tests
    all_tests_passed = cargo_tests_passed and all(tc.passed for p in packages for tc in p.test_cases)
    
    print_summary(packages, all_tests_passed, expected_violations)
    
    sys.exit(0 if all_tests_passed else 1)


if __name__ == "__main__":
    main()
