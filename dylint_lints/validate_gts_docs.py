#!/usr/bin/env python3
"""
GTS Identifier Validator for Documentation Files (DE0903)

This script validates GTS (Global Type System) identifiers in .md and .json files,
complementing the Rust-based DE0901 lint that validates GTS identifiers in source code.

GTS identifiers follow the pattern:
    gts.<vendor>.<org>.<package>.<type>.<version>~[<instance_segment>~]*

Examples:
    - Schema:   gts.x.core.modkit.plugin.v1~
    - Instance: gts.x.core.modkit.plugin.v1~vendor.pkg.my_module.plugin.v1~

Rules:
    1. Must start with "gts."
    2. Each segment must have 5 components: vendor.org.package.type.version
    3. Schema IDs must end with "~"
    4. Only lowercase letters, numbers, underscores, and dots allowed (no hyphens)
    5. Version must match pattern: v<number> or v<number>.<number>
    6. Wildcards (*) only allowed in documented pattern contexts

Usage:
    python dylint_lints/validate_gts_docs.py [--verbose] [--json] [paths...]

Exit codes:
    0 - All GTS identifiers are valid
    1 - Invalid GTS identifiers found
"""

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import List, Optional, Tuple

# Project root (parent of scripts/)
PROJECT_ROOT = Path(__file__).parent.parent

# Directories to scan by default
DEFAULT_SCAN_DIRS = [
    "docs",
    "modules",
    "libs",
    "examples",
]

# File patterns to scan
FILE_PATTERNS = ["*.md", "*.json"]

# Directories to skip
SKIP_DIRS = {
    "target",
    "node_modules",
    ".git",
    "vendor",
}

# Files to skip (relative to project root)
SKIP_FILES = {
    "docs/api/api.json",  # Generated OpenAPI spec, may have different patterns
}

# Patterns that look like GTS but aren't (false positives)
FALSE_POSITIVE_PATTERNS = [
    re.compile(r'^gts\.rs$'),           # Rust file named gts.rs
    re.compile(r'^gts\.[a-z]+$'),       # Single component like gts.rs, gts.py
]

# Pattern to find GTS-looking strings (must have at least 2 dots after gts.)
# Include hyphen so we can catch and report invalid hyphens in segments
GTS_PATTERN = re.compile(r'gts\.[a-z0-9_.*~-]+\.[a-z0-9_.*~-]+', re.IGNORECASE)

# Valid segment pattern: vendor.org.package.type.version
# Version can be: v1, v2, v1.0, v1.2.3, etc.
SEGMENT_PATTERN = re.compile(
    r'^[a-z0-9_]+\.[a-z0-9_]+\.[a-z0-9_]+\.[a-z0-9_]+\.v[0-9]+(\.[0-9]+)*$'
)

# Contexts where wildcards are allowed (in documentation)
WILDCARD_ALLOWED_CONTEXTS = [
    "pattern",
    "filter",
    "query",
    "$filter",
    "starts_with",
    "with_pattern",
    "resource_pattern",
    "discovery",
    "match",
    "wildcard",      # Describing wildcard behavior
    "differs from",  # Comparing with wildcard
    "get",           # GET requests with wildcards
    "list",          # List queries
    "todo",          # TODO items often have patterns
    "p1 -",          # Priority markers in tasks
    "p2 -",
    "p3 -",
]

# Contexts that indicate "bad example" or intentionally invalid identifiers
# These are often used in documentation to show what NOT to do
SKIP_VALIDATION_CONTEXTS = [
    "invalid",
    "wrong",
    "bad",
    "reject",
    "error",
    "fail",
    "❌",
    "✗",
    "should not",
    "must not",
    "not allowed",
    "given**",  # "GIVEN** the identifier" in spec docs
    "**given**",
]


@dataclass
class GtsError:
    """Represents a single GTS validation error"""
    file: Path
    line: int
    column: int
    gts_id: str
    error: str
    context: str = ""


@dataclass
class ValidationResult:
    """Aggregated validation results"""
    errors: List[GtsError] = field(default_factory=list)
    files_scanned: int = 0
    
    @property
    def ok(self) -> bool:
        return len(self.errors) == 0


def validate_gts_segment(segment: str) -> Tuple[bool, str]:
    """
    Validate a single GTS segment like 'x.core.modkit.plugin.v1'
    
    Returns (is_valid, error_message)
    """
    if not segment:
        return True, ""  # Empty segments are ok (trailing ~)
    
    # Check for invalid characters
    if '-' in segment:
        return False, f"Hyphen not allowed in segment: '{segment}'"
    
    # Check segment structure
    if not SEGMENT_PATTERN.match(segment):
        parts = segment.split('.')
        if len(parts) < 5:
            return False, f"Segment must have 5 components (vendor.org.package.type.version), got {len(parts)}: '{segment}'"
        if not parts[-1].startswith('v'):
            return False, f"Segment must end with version (v1, v2, etc.): '{segment}'"
        return False, f"Invalid segment format: '{segment}'"
    
    return True, ""


def validate_gts_id(gts_id: str, allow_wildcards: bool = False) -> List[str]:
    """
    Validate a complete GTS identifier.
    
    Returns a list of error messages (empty if valid).
    """
    errors = []
    original = gts_id
    
    # Normalize: remove quotes if present
    gts_id = gts_id.strip().strip('"\'')
    
    if not gts_id.startswith("gts."):
        return [f"Must start with 'gts.': '{original}'"]
    
    # Check for wildcards
    if '*' in gts_id and not allow_wildcards:
        errors.append(f"Wildcards not allowed outside pattern contexts: '{original}'")
        return errors
    
    # If wildcards are present and allowed, we do basic structure check only
    if '*' in gts_id:
        # Just check it starts with gts. and has some structure
        return []
    
    # Remove 'gts.' prefix
    rest = gts_id[4:]
    
    # Split by ~ to get segments
    segments = rest.split('~')
    
    # Filter out empty segments (from trailing ~)
    non_empty_segments = [s for s in segments if s]
    
    if not non_empty_segments:
        errors.append(f"No segments found after 'gts.': '{original}'")
        return errors
    
    for seg in non_empty_segments:
        valid, err = validate_gts_segment(seg)
        if not valid:
            errors.append(err)
    
    # Schema IDs (single segment) must end with ~
    # Instance IDs (chained) typically end with ~ but error codes may not
    if len(non_empty_segments) == 1 and not gts_id.endswith('~'):
        errors.append(f"Schema ID must end with '~': '{original}'")
    
    return errors


def is_wildcard_context(line: str, match_start: int) -> bool:
    """
    Check if the GTS identifier is in a context where wildcards are allowed.
    
    This looks for keywords before the match position that indicate
    pattern/filter context.
    """
    # Get the portion of line before the match
    before = line[:match_start].lower()
    
    # Check for wildcard-allowing contexts
    for ctx in WILDCARD_ALLOWED_CONTEXTS:
        if ctx in before:
            return True
    
    # Also check for code fence contexts that indicate filter examples
    if '$filter' in line.lower():
        return True
    
    return False


def is_bad_example_context(line: str, _match_start: int, prev_lines: Optional[List[str]] = None) -> bool:
    """
    Check if the GTS identifier is in a "bad example" context.
    
    Documentation often shows intentionally invalid identifiers to demonstrate
    what NOT to do. We should skip validation for these.
    """
    line_lower = line.lower()
    
    # Check current line for skip contexts
    for ctx in SKIP_VALIDATION_CONTEXTS:
        if ctx in line_lower:
            return True
    
    # Check a few previous lines for context (e.g., "Example: Bad" headers)
    if prev_lines:
        for prev_line in prev_lines[-3:]:  # Check last 3 lines
            prev_lower = prev_line.lower()
            for ctx in SKIP_VALIDATION_CONTEXTS:
                if ctx in prev_lower:
                    return True
    
    return False


def scan_file(path: Path, verbose: bool = False) -> List[GtsError]:
    """
    Scan a file for GTS identifiers and validate them.
    
    Returns a list of GtsError for any invalid identifiers.
    """
    errors = []
    
    try:
        content = path.read_text(encoding='utf-8')
    except (OSError, UnicodeDecodeError) as e:
        if verbose:
            print(f"  Warning: Could not read {path}: {e}")
        return errors
    
    lines = content.splitlines()
    
    for line_num, line in enumerate(lines, 1):
        for match in GTS_PATTERN.finditer(line):
            gts_id = match.group()
            col = match.start() + 1
            
            # Skip if this is in a "bad example" context
            prev_lines = lines[max(0, line_num-4):line_num-1]
            if is_bad_example_context(line, match.start(), prev_lines):
                continue
            
            # Check if wildcards are allowed in this context
            allow_wildcards = is_wildcard_context(line, match.start())
            
            # Validate the identifier
            validation_errors = validate_gts_id(gts_id, allow_wildcards=allow_wildcards)
            
            for err in validation_errors:
                # Extract some context
                context_start = max(0, match.start() - 20)
                context_end = min(len(line), match.end() + 20)
                context = line[context_start:context_end]
                if context_start > 0:
                    context = "..." + context
                if context_end < len(line):
                    context = context + "..."
                
                errors.append(GtsError(
                    file=path,
                    line=line_num,
                    column=col,
                    gts_id=gts_id,
                    error=err,
                    context=context,
                ))
    
    return errors


def find_files(paths: List[Path], verbose: bool = False) -> List[Path]:
    """
    Find all .md and .json files in the given paths.
    """
    files = []
    
    for path in paths:
        if path.is_file():
            if any(path.match(pat) for pat in FILE_PATTERNS):
                files.append(path)
        elif path.is_dir():
            for pattern in FILE_PATTERNS:
                for f in path.rglob(pattern):
                    # Skip directories
                    if any(skip in f.parts for skip in SKIP_DIRS):
                        continue
                    # Skip specific files (only if within project root)
                    if f.is_relative_to(PROJECT_ROOT):
                        rel_path = str(f.relative_to(PROJECT_ROOT))
                        if rel_path in SKIP_FILES:
                            if verbose:
                                print(f"  Skipping: {rel_path}")
                            continue
                    files.append(f)
    
    return sorted(set(files))


def validate_files(paths: List[Path], verbose: bool = False) -> ValidationResult:
    """
    Validate GTS identifiers in all specified files.
    """
    result = ValidationResult()
    
    files = find_files(paths, verbose)
    result.files_scanned = len(files)
    
    for file_path in files:
        if verbose:
            print(f"  Scanning: {file_path}")
        
        file_errors = scan_file(file_path, verbose)
        result.errors.extend(file_errors)
    
    return result


def print_results(result: ValidationResult, verbose: bool = False):
    """
    Print validation results in a formatted way.
    """
    print()
    print("=" * 80)
    print("  GTS DOCUMENTATION VALIDATOR (DE0903)")
    print("=" * 80)
    print()
    print(f"  Files scanned: {result.files_scanned}")
    print(f"  Errors found:  {len(result.errors)}")
    print()
    
    if result.errors:
        print("-" * 80)
        print("  ERRORS")
        print("-" * 80)
        
        # Group errors by file
        errors_by_file = {}
        for err in result.errors:
            if err.file not in errors_by_file:
                errors_by_file[err.file] = []
            errors_by_file[err.file].append(err)
        
        for file_path, file_errors in sorted(errors_by_file.items()):
            rel_path = file_path.relative_to(PROJECT_ROOT) if file_path.is_relative_to(PROJECT_ROOT) else file_path
            print(f"\n  {rel_path}:")
            
            for err in sorted(file_errors, key=lambda e: e.line):
                print(f"    Line {err.line}:{err.column} - {err.gts_id}")
                print(f"      Error: {err.error}")
                if verbose and err.context:
                    print(f"      Context: {err.context}")
        
        print()
    
    print("=" * 80)
    if result.ok:
        print("  STATUS: ALL GTS IDENTIFIERS VALID ✓")
    else:
        print(f"  STATUS: {len(result.errors)} INVALID GTS IDENTIFIERS FOUND ✗")
        print()
        print("  To fix:")
        print("    - Schema IDs must end with ~ (e.g., gts.x.core.type.v1~)")
        print("    - Each segment needs 5 parts: vendor.org.package.type.version")
        print("    - No hyphens allowed, use underscores")
        print("    - Wildcards (*) only in filter/pattern contexts")
    print("=" * 80)


def main():
    parser = argparse.ArgumentParser(
        description="Validate GTS identifiers in .md and .json files (DE0903)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    parser.add_argument(
        "paths",
        nargs="*",
        type=Path,
        help="Paths to scan (files or directories). Defaults to docs/, modules/, libs/, examples/",
    )
    parser.add_argument(
        "-v", "--verbose",
        action="store_true",
        help="Show verbose output including file scanning progress",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output results as JSON",
    )
    
    args = parser.parse_args()
    
    # Default paths if none specified
    if not args.paths:
        args.paths = [PROJECT_ROOT / d for d in DEFAULT_SCAN_DIRS if (PROJECT_ROOT / d).exists()]
    
    if args.verbose:
        print(f"Scanning paths: {[str(p) for p in args.paths]}")
    
    result = validate_files(args.paths, verbose=args.verbose)
    
    if args.json:
        output = {
            "files_scanned": result.files_scanned,
            "errors_count": len(result.errors),
            "ok": result.ok,
            "errors": [
                {
                    "file": str(e.file),
                    "line": e.line,
                    "column": e.column,
                    "gts_id": e.gts_id,
                    "error": e.error,
                }
                for e in result.errors
            ],
        }
        print(json.dumps(output, indent=2))
    else:
        print_results(result, verbose=args.verbose)
    
    sys.exit(0 if result.ok else 1)


if __name__ == "__main__":
    main()
