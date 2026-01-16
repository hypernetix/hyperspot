#!/usr/bin/env python3
"""
Unit tests for the GTS Documentation Validator (DE0903).

Run with:
    python -m pytest dylint_lints/test_validate_gts_docs.py -v
    
Or directly:
    python dylint_lints/test_validate_gts_docs.py
"""

import sys
import tempfile
import unittest
from pathlib import Path

# Import the validator module
sys.path.insert(0, str(Path(__file__).parent))
from validate_gts_docs import (
    validate_gts_segment,
    validate_gts_id,
    is_wildcard_context,
    is_bad_example_context,
    scan_file,
    GTS_PATTERN,
)


class TestGtsSegmentValidation(unittest.TestCase):
    """Test the validate_gts_segment function."""

    def test_valid_segment_standard(self):
        """Standard 5-component segment should be valid."""
        valid, err = validate_gts_segment("x.core.modkit.plugin.v1")
        self.assertTrue(valid, f"Expected valid, got error: {err}")

    def test_valid_segment_with_underscores(self):
        """Segment with underscores should be valid."""
        valid, err = validate_gts_segment("my_vendor.my_org.my_package.my_type.v1")
        self.assertTrue(valid, f"Expected valid, got error: {err}")

    def test_valid_segment_version_with_minor(self):
        """Segment with minor version should be valid."""
        valid, err = validate_gts_segment("x.core.modkit.plugin.v1.2")
        self.assertTrue(valid, f"Expected valid, got error: {err}")

    def test_valid_segment_version_with_patch(self):
        """Segment with patch version should be valid."""
        valid, err = validate_gts_segment("x.core.modkit.plugin.v1.2.3")
        self.assertTrue(valid, f"Expected valid, got error: {err}")

    def test_valid_segment_numeric_parts(self):
        """Segment with numeric parts should be valid."""
        valid, err = validate_gts_segment("vendor1.org2.pkg3.type4.v1")
        self.assertTrue(valid, f"Expected valid, got error: {err}")

    def test_empty_segment_is_valid(self):
        """Empty segment should be valid (trailing ~)."""
        valid, err = validate_gts_segment("")
        self.assertTrue(valid, f"Expected valid for empty segment, got error: {err}")

    def test_invalid_segment_hyphen(self):
        """Segment with hyphen should be invalid."""
        valid, err = validate_gts_segment("my-vendor.org.pkg.type.v1")
        self.assertFalse(valid)
        self.assertIn("Hyphen", err)

    def test_invalid_segment_missing_version(self):
        """Segment without version prefix should be invalid."""
        valid, _err = validate_gts_segment("x.core.modkit.plugin.1")
        self.assertFalse(valid)

    def test_invalid_segment_too_few_components(self):
        """Segment with less than 5 components should be invalid."""
        valid, err = validate_gts_segment("x.core.plugin.v1")
        self.assertFalse(valid)
        self.assertIn("5 components", err)

    def test_invalid_segment_three_components(self):
        """Segment with only 3 components should be invalid."""
        valid, err = validate_gts_segment("x.core.v1")
        self.assertFalse(valid)
        self.assertIn("5 components", err)

    def test_invalid_segment_too_many_components(self):
        """Segment with more than 5 components (not version parts) should be invalid."""
        valid, _err = validate_gts_segment("x.core.extra.modkit.plugin.v1")
        self.assertFalse(valid)


class TestGtsIdValidation(unittest.TestCase):
    """Test the validate_gts_id function."""

    def test_valid_schema_id(self):
        """Valid schema ID (ends with ~) should pass."""
        errors = validate_gts_id("gts.x.core.modkit.plugin.v1~")
        self.assertEqual(errors, [], f"Unexpected errors: {errors}")

    def test_valid_instance_id_single_chain(self):
        """Valid instance ID with one chain should pass."""
        errors = validate_gts_id("gts.x.core.modkit.plugin.v1~vendor.pkg.module.impl.v1~")
        self.assertEqual(errors, [], f"Unexpected errors: {errors}")

    def test_valid_instance_id_double_chain(self):
        """Valid instance ID with two chains should pass."""
        errors = validate_gts_id("gts.x.core.modkit.plugin.v1~vendor.pkg.module.impl.v1~vendor.ext.extra.more.v2~")
        self.assertEqual(errors, [], f"Unexpected errors: {errors}")

    def test_valid_error_code(self):
        """Valid error code format should pass."""
        errors = validate_gts_id("gts.hx.core.errors.err.v1~hx.odata.errors.invalid.v1")
        self.assertEqual(errors, [], f"Unexpected errors: {errors}")

    def test_valid_with_quotes(self):
        """GTS ID with surrounding quotes should be normalized and pass."""
        errors = validate_gts_id('"gts.x.core.modkit.plugin.v1~"')
        self.assertEqual(errors, [], f"Unexpected errors: {errors}")

    def test_valid_with_single_quotes(self):
        """GTS ID with single quotes should be normalized and pass."""
        errors = validate_gts_id("'gts.x.core.modkit.plugin.v1~'")
        self.assertEqual(errors, [], f"Unexpected errors: {errors}")

    def test_invalid_no_gts_prefix(self):
        """ID without gts. prefix should fail."""
        errors = validate_gts_id("x.core.modkit.plugin.v1~")
        self.assertGreater(len(errors), 0)
        self.assertIn("gts.", errors[0])

    def test_invalid_wildcard_without_context(self):
        """Wildcard without pattern context should fail."""
        errors = validate_gts_id("gts.x.*.modkit.plugin.v1~", allow_wildcards=False)
        self.assertGreater(len(errors), 0)
        self.assertIn("Wildcard", errors[0])

    def test_valid_wildcard_with_context(self):
        """Wildcard with allow_wildcards=True should pass."""
        errors = validate_gts_id("gts.x.*.modkit.plugin.v1~", allow_wildcards=True)
        self.assertEqual(errors, [], f"Unexpected errors: {errors}")

    def test_invalid_segment_too_few_parts(self):
        """Segment with too few parts should fail."""
        errors = validate_gts_id("gts.x.core.plugin.v1~")
        self.assertGreater(len(errors), 0)
        self.assertIn("5 components", errors[0])

    def test_invalid_hyphen_in_segment(self):
        """Hyphen in segment should fail."""
        errors = validate_gts_id("gts.my-vendor.core.modkit.plugin.v1~")
        self.assertGreater(len(errors), 0)
        self.assertIn("Hyphen", errors[0])

    def test_invalid_no_segments(self):
        """GTS with no segments should fail."""
        errors = validate_gts_id("gts.~")
        self.assertGreater(len(errors), 0)

    def test_invalid_schema_without_trailing_tilde(self):
        """Schema ID (single segment) without trailing ~ should fail."""
        errors = validate_gts_id("gts.x.core.modkit.plugin.v1")
        self.assertGreater(len(errors), 0)
        self.assertIn("must end with '~'", errors[0])


class TestWildcardContext(unittest.TestCase):
    """Test the is_wildcard_context function."""

    def test_filter_context(self):
        """$filter context should allow wildcards."""
        line = "$filter=type_id eq 'gts.x.*'"
        match_start = line.find("gts")
        self.assertTrue(is_wildcard_context(line, match_start))

    def test_pattern_context(self):
        """Pattern context should allow wildcards."""
        line = "Use this pattern: gts.x.core.*.*.v1~"
        match_start = line.find("gts")
        self.assertTrue(is_wildcard_context(line, match_start))

    def test_with_pattern_method(self):
        """.with_pattern() context should allow wildcards."""
        line = '.with_pattern("gts.x.core.*")'
        match_start = line.find("gts")
        self.assertTrue(is_wildcard_context(line, match_start))

    def test_resource_pattern_context(self):
        """resource_pattern context should allow wildcards."""
        line = '.resource_pattern("gts.x.core.type.v1~*")'
        match_start = line.find("gts")
        self.assertTrue(is_wildcard_context(line, match_start))

    def test_query_context(self):
        """Query context should allow wildcards."""
        line = "Query for gts.vendor.* to get all types"
        match_start = line.find("gts")
        self.assertTrue(is_wildcard_context(line, match_start))

    def test_no_context(self):
        """Normal context should not allow wildcards."""
        line = "The type gts.x.core.type.v1~ is defined here"
        match_start = line.find("gts")
        self.assertFalse(is_wildcard_context(line, match_start))


class TestBadExampleContext(unittest.TestCase):
    """Test the is_bad_example_context function."""

    def test_invalid_keyword(self):
        """Line with 'invalid' should be skipped."""
        line = "Invalid: gts.bad.id"
        self.assertTrue(is_bad_example_context(line, 0))

    def test_wrong_keyword(self):
        """Line with 'wrong' should be skipped."""
        line = "This is wrong: gts.x.bad"
        self.assertTrue(is_bad_example_context(line, 0))

    def test_bad_keyword(self):
        """Line with 'bad' should be skipped."""
        line = "Bad example: gts.x.y.z"
        self.assertTrue(is_bad_example_context(line, 0))

    def test_x_emoji(self):
        """Line with ❌ should be skipped."""
        line = "❌ gts.x.y.z.a.v1~"
        self.assertTrue(is_bad_example_context(line, 0))

    def test_reject_keyword(self):
        """Line with 'reject' should be skipped."""
        line = "Should reject gts.invalid.id"
        self.assertTrue(is_bad_example_context(line, 0))

    def test_error_keyword(self):
        """Line with 'error' should be skipped."""
        line = "This causes an error: gts.x"
        self.assertTrue(is_bad_example_context(line, 0))

    def test_previous_line_context(self):
        """Bad example context in previous lines should be detected."""
        line = "gts.x.y.z~"
        prev_lines = ["Example: Bad", "", ""]
        self.assertTrue(is_bad_example_context(line, 0, prev_lines))

    def test_normal_context(self):
        """Normal context should not be skipped."""
        line = "The correct format is gts.x.core.type.v1~"
        self.assertFalse(is_bad_example_context(line, 0))


class TestGtsPattern(unittest.TestCase):
    """Test the GTS_PATTERN regex."""

    def test_matches_valid_gts(self):
        """Pattern should match valid GTS IDs."""
        text = "gts.x.core.modkit.plugin.v1~"
        match = GTS_PATTERN.search(text)
        self.assertIsNotNone(match)

    def test_matches_in_context(self):
        """Pattern should match GTS ID in surrounding text."""
        text = "The type is `gts.x.core.modkit.plugin.v1~` and more"
        match = GTS_PATTERN.search(text)
        self.assertIsNotNone(match)
        self.assertEqual(match.group(), "gts.x.core.modkit.plugin.v1~")

    def test_no_match_single_component(self):
        """Pattern should not match single component after gts."""
        text = "gts.rs"
        match = GTS_PATTERN.search(text)
        self.assertIsNone(match)

    def test_no_match_gts_alone(self):
        """Pattern should not match 'gts' alone."""
        text = "gts is an acronym"
        match = GTS_PATTERN.search(text)
        self.assertIsNone(match)


class TestScanFile(unittest.TestCase):
    """Test the scan_file function with actual files."""

    def test_valid_markdown_file(self):
        """File with valid GTS should have no errors."""
        content = """# Documentation

The main type is `gts.x.core.modkit.plugin.v1~` which implements the plugin.

## Instance

An instance: `gts.x.core.modkit.plugin.v1~vendor.pkg.module.impl.v1~`
"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False) as f:
            f.write(content)
            f.flush()
            path = Path(f.name)
        
        try:
            errors = scan_file(path)
            self.assertEqual(len(errors), 0, f"Unexpected errors: {errors}")
        finally:
            path.unlink()

    def test_invalid_segment_detected(self):
        """File with invalid segment should report error."""
        content = """# Documentation

The type is: `gts.x.core.plugin.v1~`
"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False) as f:
            f.write(content)
            f.flush()
            path = Path(f.name)
        
        try:
            errors = scan_file(path)
            self.assertEqual(len(errors), 1)
            self.assertIn("5 components", errors[0].error)
        finally:
            path.unlink()

    def test_bad_example_skipped(self):
        """Invalid GTS in 'bad example' context should be skipped."""
        content = """# Documentation

## Example: Bad

Invalid format: `gts.x.core.v1~`

## Example: Good

Valid format: `gts.x.core.modkit.plugin.v1~`
"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False) as f:
            f.write(content)
            f.flush()
            path = Path(f.name)
        
        try:
            errors = scan_file(path)
            self.assertEqual(len(errors), 0, f"Unexpected errors: {errors}")
        finally:
            path.unlink()

    def test_wildcard_in_filter_allowed(self):
        """Wildcard in $filter context should be allowed."""
        content = """# API

Use `$filter=type_id eq 'gts.x.*'` to filter by vendor.
"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False) as f:
            f.write(content)
            f.flush()
            path = Path(f.name)
        
        try:
            errors = scan_file(path)
            self.assertEqual(len(errors), 0, f"Unexpected errors: {errors}")
        finally:
            path.unlink()

    def test_wildcard_without_context_fails(self):
        """Wildcard without pattern context should fail."""
        content = """# Types

The type is `gts.x.*.modkit.plugin.v1~`
"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False) as f:
            f.write(content)
            f.flush()
            path = Path(f.name)
        
        try:
            errors = scan_file(path)
            self.assertEqual(len(errors), 1)
            self.assertIn("Wildcard", errors[0].error)
        finally:
            path.unlink()

    def test_hyphen_detected(self):
        """Hyphen in segment should be detected and reported as error."""
        content = """# Types

Type: `gts.my-vendor.core.modkit.plugin.v1~`
"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False) as f:
            f.write(content)
            f.flush()
            path = Path(f.name)
        
        try:
            errors = scan_file(path)
            # The hyphen should be matched and flagged as invalid
            self.assertEqual(len(errors), 1)
            self.assertIn("Hyphen", errors[0].error)
        finally:
            path.unlink()

    def test_json_file_validation(self):
        """JSON files should be validated."""
        content = """{
  "schema_id": "gts.x.core.modkit.plugin.v1~",
  "other_type": "gts.x.core.oops.v1~"
}"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            f.write(content)
            f.flush()
            path = Path(f.name)
        
        try:
            errors = scan_file(path)
            # Should find one error for the 4-component segment
            self.assertEqual(len(errors), 1)
            self.assertIn("5 components", errors[0].error)
        finally:
            path.unlink()

    def test_chained_instance_validation(self):
        """Chained instance IDs should validate each segment."""
        content = """# Instance

`gts.x.core.modkit.plugin.v1~vendor.pkg.oops.v1~`
"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False) as f:
            f.write(content)
            f.flush()
            path = Path(f.name)
        
        try:
            errors = scan_file(path)
            self.assertEqual(len(errors), 1)
            self.assertIn("5 components", errors[0].error)
        finally:
            path.unlink()


class TestEdgeCases(unittest.TestCase):
    """Test edge cases and boundary conditions."""

    def test_multiple_gts_on_one_line(self):
        """Multiple GTS IDs on one line should all be validated."""
        content = """Compare `gts.x.core.modkit.plugin.v1~` with `gts.y.core.modkit.other.v1~`
"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False) as f:
            f.write(content)
            f.flush()
            path = Path(f.name)
        
        try:
            errors = scan_file(path)
            self.assertEqual(len(errors), 0, f"Unexpected errors: {errors}")
        finally:
            path.unlink()

    def test_gts_in_code_block(self):
        """GTS in code blocks should be validated."""
        content = """```rust
let schema_id = "gts.x.core.modkit.plugin.v1~";
```
"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False) as f:
            f.write(content)
            f.flush()
            path = Path(f.name)
        
        try:
            errors = scan_file(path)
            self.assertEqual(len(errors), 0, f"Unexpected errors: {errors}")
        finally:
            path.unlink()

    def test_version_formats(self):
        """Various version formats should be valid."""
        test_cases = [
            ("gts.x.core.modkit.plugin.v1~", True),
            ("gts.x.core.modkit.plugin.v2~", True),
            ("gts.x.core.modkit.plugin.v10~", True),
            ("gts.x.core.modkit.plugin.v1.0~", True),
            ("gts.x.core.modkit.plugin.v1.2.3~", True),
        ]
        
        for gts_id, expected_valid in test_cases:
            with self.subTest(gts_id=gts_id):
                errors = validate_gts_id(gts_id)
                if expected_valid:
                    self.assertEqual(errors, [], f"Expected valid: {gts_id}, got errors: {errors}")
                else:
                    self.assertGreater(len(errors), 0, f"Expected invalid: {gts_id}")

    def test_lowercase_requirement(self):
        """GTS IDs must be lowercase."""
        # The SEGMENT_PATTERN only matches lowercase
        valid, _ = validate_gts_segment("X.Core.Modkit.Plugin.v1")
        self.assertFalse(valid)


if __name__ == '__main__':
    # Run with verbose output
    unittest.main(verbosity=2)
