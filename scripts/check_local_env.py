#!/usr/bin/env python3
"""
Test environment prerequisite checker.

This script checks all available prerequisites and prints their status in ASCII format.
Exits with 0 if all prerequisites pass, 1 if any fail.
"""

import sys
import logging
import os
import argparse
from typing import List

# Add the scripts directory to the path so we can import our modules
sys.path.insert(0, os.path.dirname(__file__))

from lib.prereq import (
    ALL_PREREQS, CORE_PREREQS, E2E_LOCAL_PREREQS,
    PRECHECK_OK, PRECHECK_WARNING, check_prerequisites
)


def print_ascii_header():
    """Print ASCII header"""
    print("=" * 80)
    print("  HYPERSPOT TEST ENVIRONMENT PREREQUISITES CHECK")
    print("=" * 80)
    print()


def print_ascii_footer(passed: int, total: int, prereqs_with_remediation: List[str]):
    """Print ASCII footer with results"""
    print()
    print("=" * 80)
    print(f"  RESULTS: {passed}/{total} prerequisites passed")
    if passed == total:
        print("  STATUS: ALL PREREQUISITES OK")
    else:
        print(f"  STATUS: {total - passed} PREREQUISITES FAILED")
        print("\n  REMEDIATION INSTRUCTIONS:")

        n = 1
        for prereq in prereqs_with_remediation:
            print(f"\n   {n}. Problem: '{prereq.name}' check failed\n      Possible remediation: {prereq.remediation}")
            n += 1
    print("=" * 80)


def check_all_prereqs(prereq_list=None) -> bool:
    """
    Check all prerequisites and return True if all pass, False otherwise.

    Args:
        prereq_list: List of prerequisite classes to check. Defaults to ALL_PREREQS.
    """
    # Configure logging to suppress debug/info messages during checks
    logging.basicConfig(level=logging.ERROR, format='%(levelname)s: %(message)s')

    print_ascii_header()

    if prereq_list is None:
        prereq_list = ALL_PREREQS

    passed = 0
    total = len(prereq_list)
    prereqs_with_remediation = []

    for prereq_class in prereq_list:
        prereq = prereq_class()
        prereq_name = prereq.name

        # Print the prerequisite name with padding
        print(f"  {prereq_name:<55} ... ", end="", flush=True)

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
                prereqs_with_remediation.append(prereq)

            print(result)

        except Exception as e:
            print(f"ERROR: {str(e)}")
            prereqs_with_remediation.append(prereq)

    print_ascii_footer(passed, total, prereqs_with_remediation)

    return passed == total


def main():
    """Main function"""
    parser = argparse.ArgumentParser(
        description="Check HyperSpot test environment prerequisites"
    )
    parser.add_argument(
        "--mode", "-m",
        choices=["core", "e2e-local", "full"],
        default="full",
        help="Type of prerequisites to check (default: full)"
    )
    parser.add_argument(
        "--quiet", "-q",
        action="store_true",
        help="Only output results, no detailed checks"
    )

    args = parser.parse_args()

    # Select the appropriate prerequisite list based on mode
    if args.mode == "core":
        prereq_list = CORE_PREREQS
        mode_desc = "core testing prerequisites"
    elif args.mode == "e2e-local":
        prereq_list = E2E_LOCAL_PREREQS
        mode_desc = "e2e local testing prerequisites"
    else:
        prereq_list = ALL_PREREQS
        mode_desc = "all testing prerequisites"

    if args.quiet:
        # Quiet mode: just check and return exit code
        passed, total, failed_prereqs = check_prerequisites(prereq_list)
        if failed_prereqs:
            print(f"Environment not ready: {total - passed}/{total} prerequisites failed")
            for prereq in failed_prereqs:
                print(f"  - {prereq.name}")
            sys.exit(1)
        else:
            print(f"Environment ready: {passed}/{total} prerequisites passed")
            sys.exit(0)
    else:
        # Verbose mode: show detailed checks
        all_passed = check_all_prereqs(prereq_list)

        if all_passed:
            print(f"\nAll {mode_desc} passed! Environment is ready for '{args.mode}' testing.")
            sys.exit(0)
        else:
            print(f"\nSome {mode_desc} failed. Please fix the issues above before running tests.")
            sys.exit(1)


if __name__ == "__main__":
    main()
