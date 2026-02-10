"""
Generate golden Markdown reference files for file-parser E2E tests.

This script:
1. Scans e2e/testdata/ for test input files (PDFs, DOCX, etc.)
2. Uploads each file to the /file-parser/v1/upload endpoint with render_markdown=true
3. Saves the markdown response to e2e/testdata/md/<name>.md

Usage:
    python -m e2e.modules.file_parser.generate_file_parser_golden

Environment Variables:
    E2E_BASE_URL: Base URL for the API (default: http://127.0.0.1:8086)
    E2E_AUTH_TOKEN: Optional bearer token for authentication
"""

import os
import sys
from pathlib import Path
import httpx


def get_base_url():
    """Get base URL from environment or use default."""
    return os.getenv("E2E_BASE_URL", "http://127.0.0.1:8086")


def get_auth_headers():
    """Get authorization headers if token is set."""
    token = os.getenv("E2E_AUTH_TOKEN")
    if token:
        return {"Authorization": f"Bearer {token}"}
    return {}


def find_input_files(testdata_dir):
    """
    Find all input test files in testdata directory.

    Looks for files in testdata/docx/ and testdata/pdf/ subdirectories.
    Excludes .txt, .text, and .md files (those are reference files).

    Returns:
        List of Path objects for input files
    """
    input_files = []

    # Scan docx subdirectory
    docx_dir = testdata_dir / "docx"
    if docx_dir.exists():
        for file_path in docx_dir.iterdir():
            if file_path.is_file() and file_path.suffix.lower() == ".docx":
                input_files.append(file_path)

    # Scan pdf subdirectory
    pdf_dir = testdata_dir / "pdf"
    if pdf_dir.exists():
        for file_path in pdf_dir.iterdir():
            if file_path.is_file() and file_path.suffix.lower() == ".pdf":
                input_files.append(file_path)

    return sorted(input_files)


def generate_golden_markdown(input_file, base_url, headers, md_output_dir):
    """
    Generate golden markdown for a single input file.

    Args:
        input_file: Path to input file
        base_url: API base URL
        headers: Auth headers dict
        md_output_dir: Directory to write golden markdown files

    Returns:
        bool: True if successful, False otherwise
    """
    print(f"Processing: {input_file.name}...", end=" ", flush=True)

    try:
        # Read file content
        with open(input_file, "rb") as f:
            file_content = f.read()

        # Call API endpoint
        url = f"{base_url}/file-parser/v1/upload"
        params = {
            "render_markdown": "true",
            "filename": input_file.name
        }

        with httpx.Client(timeout=30.0) as client:
            response = client.post(
                url,
                params=params,
                headers={**headers, "Content-Type": "application/octet-stream"},
                content=file_content
            )

        # Check response
        if response.status_code != 200:
            print(f"FAILED (HTTP {response.status_code})")
            print(f"  Response: {response.text[:200]}")
            return False

        # Parse JSON response
        data = response.json()

        if "markdown" not in data or data["markdown"] is None:
            print("FAILED (no markdown in response)")
            return False

        markdown_content = data["markdown"]

        # Determine output file path
        # Input: testdata/docx/test_file_1table.docx -> Output: testdata/md/test_file_1table.md
        output_file = md_output_dir / f"{input_file.stem}.md"

        # Write markdown to file
        output_file.write_text(markdown_content, encoding="utf-8")

        print(f"OK -> {output_file.relative_to(Path.cwd())}")
        return True

    except Exception as e:
        print(f"ERROR: {e}")
        return False


def main():
    """Main entry point for the golden markdown generator."""
    # Determine paths
    script_dir = Path(__file__).parent
    # testdata is at e2e/testdata, script is at e2e/modules/file-parser
    testdata_dir = script_dir.parent.parent / "testdata"
    md_output_dir = testdata_dir / "md"

    # Validate testdata directory exists
    if not testdata_dir.exists():
        print(f"ERROR: testdata directory not found at {testdata_dir}")
        sys.exit(1)

    # Create md output directory if needed
    md_output_dir.mkdir(exist_ok=True)

    # Get configuration
    base_url = get_base_url()
    headers = get_auth_headers()

    print("=" * 70)
    print("File Parser Golden Markdown Generator")
    print("=" * 70)
    print(f"Base URL: {base_url}")
    print(f"Auth: {'Enabled' if headers else 'Disabled'}")
    print(f"Output directory: {md_output_dir.relative_to(Path.cwd())}")
    print("=" * 70)
    print()

    # Find input files
    input_files = find_input_files(testdata_dir)

    if not input_files:
        print("No input files found in testdata directory.")
        sys.exit(0)

    print(f"Found {len(input_files)} input file(s)\n")

    # Process each file
    success_count = 0
    for input_file in input_files:
        if generate_golden_markdown(input_file, base_url, headers, md_output_dir):
            success_count += 1

    print()
    print("=" * 70)
    print(f"Complete: {success_count}/{len(input_files)} files processed successfully")
    print("=" * 70)

    if success_count < len(input_files):
        sys.exit(1)


if __name__ == "__main__":
    main()
