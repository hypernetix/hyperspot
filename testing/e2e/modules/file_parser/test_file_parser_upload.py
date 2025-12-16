"""E2E tests for /file-parser/v1/upload endpoint."""
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
async def test_upload_with_markdown_comparison(base_url, auth_headers, input_file, golden_md, test_id):
    """
    Test POST /file-parser/v1/upload endpoint with markdown rendering.

    This test:
    1. Uploads a file with render_markdown=true
    2. Compares the returned markdown with the golden reference
    3. Validates the JSON IR structure
    """
    # Read input file
    with open(input_file, "rb") as f:
        file_content = f.read()

    # Read golden markdown
    golden_markdown = golden_md.read_text(encoding="utf-8")

    # Call API endpoint
    url = f"{base_url}/file-parser/v1/upload"
    params = {
        "render_markdown": "true",
        "filename": input_file.name
    }

    async with httpx.AsyncClient(timeout=30.0) as client:
        response = await client.post(
            url,
            params=params,
            headers={**auth_headers, "Content-Type": "application/octet-stream"},
            content=file_content
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

    # Assert response is JSON
    assert response.headers.get("content-type", "").startswith("application/json"), (
        f"Response should be JSON for {test_id}"
    )

    # Parse JSON response
    data = response.json()

    # Validate response structure (ParsedDocResponseDto)
    assert "document" in data, f"Response should contain 'document' field for {test_id}"
    assert "markdown" in data, f"Response should contain 'markdown' field for {test_id}"

    # Validate markdown field
    assert data["markdown"] is not None, f"'markdown' should not be null for {test_id}"
    assert isinstance(data["markdown"], str), f"'markdown' should be a string for {test_id}"
    assert len(data["markdown"]) > 0, f"'markdown' should not be empty for {test_id}"

    # Compare markdown with golden reference
    actual_markdown = normalize_markdown(data["markdown"])
    expected_markdown = normalize_markdown(golden_markdown)

    assert actual_markdown == expected_markdown, (
        f"Markdown mismatch for {test_id}. "
        f"First difference at character {_find_first_diff(actual_markdown, expected_markdown)}"
    )

    # Validate document structure (ParsedDocumentDto)
    document = data["document"]
    assert isinstance(document, dict), f"'document' should be an object for {test_id}"

    # Check required fields
    assert "meta" in document, f"'document' should contain 'meta' field for {test_id}"
    assert "blocks" in document, f"'document' should contain 'blocks' field for {test_id}"

    # Validate blocks
    blocks = document["blocks"]
    assert isinstance(blocks, list), f"'blocks' should be a list for {test_id}"
    assert len(blocks) > 0, f"'blocks' should not be empty for {test_id}"

    # Known block types from the API spec
    known_block_types = {
        "heading", "paragraph", "list_item", "code_block",
        "table", "quote", "horizontal_rule", "image", "page_break"
    }

    # Validate a sample of blocks (first 5)
    for i, block in enumerate(blocks[:5]):
        assert isinstance(block, dict), f"Block {i} should be an object for {test_id}"
        assert "type" in block, f"Block {i} should have 'type' field for {test_id}"

        block_type = block["type"]
        assert block_type in known_block_types, (
            f"Block {i} has unknown type '{block_type}' for {test_id}. "
            f"Known types: {known_block_types}"
        )


def _find_first_diff(s1, s2):
    """Find the first character position where two strings differ."""
    for i, (c1, c2) in enumerate(zip(s1, s2)):
        if c1 != c2:
            return i
    return min(len(s1), len(s2))


@pytest.mark.asyncio
async def test_upload_without_render_markdown(base_url, auth_headers):
    """
    Test POST /file-parser/v1/upload endpoint without markdown rendering.

    This test verifies that when render_markdown is not set (or false),
    the response contains only the IR and markdown is null.
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

    # Call API endpoint without render_markdown
    url = f"{base_url}/file-parser/v1/upload"
    params = {
        "filename": input_file.name
    }

    async with httpx.AsyncClient(timeout=30.0) as client:
        response = await client.post(
            url,
            params=params,
            headers={**auth_headers, "Content-Type": "application/octet-stream"},
            content=file_content
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

    # Parse JSON response
    data = response.json()

    # Validate response structure
    assert "document" in data, "Response should contain 'document' field"

    # When render_markdown is not set, markdown field may be absent, null, or empty
    # (depending on backend implementation)
    # Just verify that document is present and valid
    document = data["document"]
    assert isinstance(document, dict), "'document' should be an object"
    assert "blocks" in document, "'document' should contain 'blocks'"
    assert len(document["blocks"]) > 0, "'blocks' should not be empty"

    # If markdown field is present, it should be null or empty
    if "markdown" in data:
        markdown = data["markdown"]
        assert markdown is None or markdown == "", (
            "When render_markdown is not set, markdown should be null or empty"
        )
