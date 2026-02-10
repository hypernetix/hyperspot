#!/usr/bin/env python3
"""
Validate module folder names follow kebab-case naming convention.

This script ensures all module directory names in modules/ follow the same
kebab-case rules enforced by the #[modkit::module] macro at compile time:
- Must contain only lowercase letters (a-z), digits (0-9), and hyphens (-)
- Must start with a lowercase letter
- Must not end with a hyphen
- Must not contain consecutive hyphens
- Must not contain underscores (use hyphens instead)

Exit codes:
  0 - All module names are valid
  1 - One or more module names violate kebab-case rules
"""

import sys
from pathlib import Path
from typing import List, Tuple


def validate_kebab_case(name: str) -> Tuple[bool, str]:
    """
    Validate that a module name follows kebab-case convention.
    
    Returns:
        (is_valid, error_message)
    """
    if not name:
        return False, "module name cannot be empty"
    
    # Check for underscores (common mistake - should be kebab-case, not snake_case)
    if "_" in name:
        suggested = name.replace("_", "-")
        return False, f"module name must use kebab-case, not snake_case\n       → use '{suggested}' instead of '{name}'"
    
    # Must start with a lowercase letter
    if not name[0].islower() or not name[0].isalpha():
        return False, f"module name must start with a lowercase letter, found '{name[0]}'"
    
    # Must not end with hyphen
    if name.endswith("-"):
        return False, "module name must not end with a hyphen"
    
    # Check for invalid characters and consecutive hyphens
    prev_was_hyphen = False
    for ch in name:
        if ch == "-":
            if prev_was_hyphen:
                return False, "module name must not contain consecutive hyphens"
            prev_was_hyphen = True
        elif ch.islower() or ch.isdigit():
            prev_was_hyphen = False
        else:
            return False, f"module name must contain only lowercase letters, digits, and hyphens, found '{ch}'"
    
    return True, ""


def find_modules(modules_dir: Path) -> List[Path]:
    """Find all module directories (direct subdirectories of modules/)."""
    if not modules_dir.exists() or not modules_dir.is_dir():
        return []
    
    modules = []
    for item in modules_dir.iterdir():
        if item.is_dir() and not item.name.startswith("."):
            modules.append(item)
    
    return sorted(modules)


def main() -> int:
    # Find workspace root (script is in scripts/, workspace root is parent)
    script_dir = Path(__file__).parent
    workspace_root = script_dir.parent
    modules_dir = workspace_root / "modules"
    
    if not modules_dir.exists():
        print(f"Error: modules/ directory not found at {modules_dir}", file=sys.stderr)
        return 1
    
    # Find all module directories
    modules = find_modules(modules_dir)
    
    if not modules:
        print("Warning: No modules found in modules/", file=sys.stderr)
        return 0
    
    # Validate each module name
    violations = []
    valid_count = 0
    
    for module_path in modules:
        module_name = module_path.name
        is_valid, error_msg = validate_kebab_case(module_name)
        
        if not is_valid:
            violations.append((module_path, error_msg))
        else:
            valid_count += 1
    
    # Report results
    if violations:
        print("=" * 80, file=sys.stderr)
        print("MODULE NAMING VIOLATIONS DETECTED", file=sys.stderr)
        print("=" * 80, file=sys.stderr)
        print(file=sys.stderr)
        print("All module folder names must follow kebab-case convention:", file=sys.stderr)
        print("  - Lowercase letters (a-z), digits (0-9), and hyphens (-) only", file=sys.stderr)
        print("  - Must start with a lowercase letter", file=sys.stderr)
        print("  - No trailing hyphens or consecutive hyphens", file=sys.stderr)
        print("  - No underscores (use hyphens instead)", file=sys.stderr)
        print(file=sys.stderr)
        print(f"Found {len(violations)} violation(s):", file=sys.stderr)
        print(file=sys.stderr)
        
        for module_path, error_msg in violations:
            rel_path = module_path.relative_to(workspace_root)
            print(f"  [X] {rel_path}/", file=sys.stderr)
            print(f"      {error_msg}", file=sys.stderr)
            print(file=sys.stderr)
        
        print("=" * 80, file=sys.stderr)
        print(f"Summary: {valid_count} valid, {len(violations)} invalid", file=sys.stderr)
        print("=" * 80, file=sys.stderr)
        return 1
    
    # All valid
    print(f"OK: All {valid_count} module names follow kebab-case convention")
    return 0


if __name__ == "__main__":
    sys.exit(main())
