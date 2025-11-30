"""E2E tests for /file-parser/upload/markdown endpoint."""
import httpx
import pytest
from pathlib import Path


def normalize_markdown(text):
    """
    Normalize markdown text for comparison.
    
    - Strips trailing whitespace on each line
    - Normalizes line endings to \\n
    - Strips leading/trailing blank lines
    """
    lines = text.replace("\r\n", "\n").replace("\r", "\n").split("\n")
    normalized = [line.rstrip() for line in lines]
    
    # Strip leading blank lines
    while normalized and not normalized[0]:
        normalized.pop(0)
    
    # Strip trailing blank lines
    while normalized and not normalized[-1]:
        normalized.pop()
    
    return "\n".join(normalized)


def find_test_file_pairs():
    """
    Find all (input_file, golden_md_file) pairs for testing.
    
    Returns:
        List of tuples: (input_file_path, golden_md_path, test_id)
    """
    testdata_dir = Path(__file__).parent.parent.parent / "testdata"
    md_dir = testdata_dir / "md"
    
    if not testdata_dir.exists():
        return []
    
    pairs = []
    
    # Scan for input files in subdirectories
    for subdir_name in ["docx", "pdf"]:
        subdir = testdata_dir / subdir_name
        if not subdir.exists():
            continue
        
        for input_file in subdir.iterdir():
            if not input_file.is_file():
                continue
            
            # Skip non-document files (reference text files)
            if input_file.suffix.lower() in [".txt", ".text", ".md"]:
                continue
            
            # Look for file-type-specific golden markdown first
            # e.g., test_file_two_pages_international_pdf.md or test_file_two_pages_international_docx.md
            file_ext = input_file.suffix.lower().lstrip('.')
            specific_golden_md = md_dir / f"{input_file.stem}_{file_ext}.md"
            
            # Fall back to generic golden markdown
            generic_golden_md = md_dir / f"{input_file.stem}.md"
            
            golden_md = None
            if specific_golden_md.exists():
                golden_md = specific_golden_md
            elif generic_golden_md.exists():
                golden_md = generic_golden_md
            
            if golden_md:
                # Create a test ID from the file name
                test_id = f"{subdir_name}/{input_file.name}"
                pairs.append((input_file, golden_md, test_id))
    
    return sorted(pairs, key=lambda x: x[2])


# Generate test parameters
test_file_pairs = find_test_file_pairs()


@pytest.mark.asyncio
@pytest.mark.parametrize("input_file,golden_md,test_id", test_file_pairs, ids=[p[2] for p in test_file_pairs])
async def test_upload_markdown_multipart(base_url, auth_headers, input_file, golden_md, test_id):
    """
    Test POST /file-parser/upload/markdown endpoint with multipart/form-data.
    
    This test:
    1. Uploads a file using multipart/form-data
    2. Expects text/markdown response
    3. Compares the returned markdown with the golden reference
    """
    # Read input file
    with open(input_file, "rb") as f:
        file_content = f.read()
    
    # Read golden markdown
    golden_markdown = golden_md.read_text(encoding="utf-8")
    
    # Call API endpoint with multipart/form-data
    url = f"{base_url}/file-parser/upload/markdown"
    
    # Prepare multipart form data
    files = {
        "file": (input_file.name, file_content, "application/octet-stream")
    }
    
    async with httpx.AsyncClient(timeout=30.0) as client:
        response = await client.post(
            url,
            headers=auth_headers,  # Don't set Content-Type; httpx will set it for multipart
            files=files
        )
    
    # Handle auth requirements
    if response.status_code in (401, 403) and not auth_headers:
        pytest.skip(
            f"Endpoint requires authentication (got {response.status_code}). "
            "Set E2E_AUTH_TOKEN environment variable to run this test."
        )
    
    # Handle server errors with helpful message
    if response.status_code >= 500:
        pytest.fail(
            f"Server error {response.status_code} for {test_id}. "
            f"Response: {response.text[:500]}"
        )
    
    # Assert successful response
    assert response.status_code == 200, (
        f"Expected 200, got {response.status_code} for {test_id}. "
        f"Response: {response.text[:500]}"
    )
    
    # Assert response is text/markdown
    content_type = response.headers.get("content-type", "")
    assert "text/markdown" in content_type or "text/plain" in content_type, (
        f"Response should be text/markdown for {test_id}, got {content_type}"
    )
    
    # Get response text
    actual_markdown = response.text
    
    # Validate markdown is not empty
    assert len(actual_markdown) > 0, f"Markdown should not be empty for {test_id}"
    
    # Compare markdown with golden reference
    actual_normalized = normalize_markdown(actual_markdown)
    expected_normalized = normalize_markdown(golden_markdown)
    
    assert actual_normalized == expected_normalized, (
        f"Markdown mismatch for {test_id}. "
        f"First difference at character {_find_first_diff(actual_normalized, expected_normalized)}"
    )


def _find_first_diff(s1, s2):
    """Find the first character position where two strings differ."""
    for i, (c1, c2) in enumerate(zip(s1, s2)):
        if c1 != c2:
            return i
    return min(len(s1), len(s2))


@pytest.mark.asyncio
async def test_upload_markdown_multipart_basic(base_url, auth_headers):
    """
    Test POST /file-parser/upload/markdown endpoint with a single file.
    
    This is a basic smoke test to verify the endpoint works without
    comparing to golden files.
    """
    testdata_dir = Path(__file__).parent.parent.parent / "testdata"
    
    # Use the first available PDF file
    pdf_dir = testdata_dir / "pdf"
    if not pdf_dir.exists():
        pytest.skip("No PDF test files available")
    
    input_files = [f for f in pdf_dir.iterdir() if f.suffix.lower() == ".pdf"]
    if not input_files:
        pytest.skip("No PDF test files available")
    
    input_file = input_files[0]
    
    # Read input file
    with open(input_file, "rb") as f:
        file_content = f.read()
    
    # Call API endpoint with multipart/form-data
    url = f"{base_url}/file-parser/upload/markdown"
    
    files = {
        "file": (input_file.name, file_content, "application/octet-stream")
    }
    
    async with httpx.AsyncClient(timeout=30.0) as client:
        response = await client.post(
            url,
            headers=auth_headers,
            files=files
        )
    
    # Handle auth requirements
    if response.status_code in (401, 403) and not auth_headers:
        pytest.skip(
            f"Endpoint requires authentication (got {response.status_code}). "
            "Set E2E_AUTH_TOKEN environment variable to run this test."
        )
    
    # Assert successful response
    assert response.status_code == 200, (
        f"Expected 200, got {response.status_code}. "
        f"Response: {response.text[:500]}"
    )
    
    # Validate response is not empty
    assert len(response.text) > 0, "Response should not be empty"
    
    # Basic check that it looks like markdown (contains some common markdown patterns)
    text = response.text
    # Just verify it's text and has some content
    assert isinstance(text, str), "Response should be a string"

