"""E2E tests for /file-parser/v1/parse-url and /file-parser/v1/parse-url/markdown endpoints."""
import httpx
import pytest
from pathlib import Path
import sys

# Add helpers to path
sys.path.insert(0, str(Path(__file__).parent / "helpers"))
from mock_server import mock_url


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
    Find all (relative_path, golden_md_file) pairs for testing.

    Returns:
        List of tuples: (relative_path, golden_md_path, test_id)
    """
    testdata_dir = Path(__file__).parent.parent.parent / "testdata"
    md_dir = testdata_dir / "md"

    if not testdata_dir.exists():
        return []

    pairs = []

    # Scan for input files in subdirectories (docx, pdf)
    for subdir_name in ["docx", "pdf"]:
        subdir = testdata_dir / subdir_name
        if not subdir.exists():
            continue

        for input_file in subdir.iterdir():
            if not input_file.is_file():
                continue

            # Skip non-document files
            if input_file.suffix.lower() in [".txt", ".text", ".md"]:
                continue

            # Look for file-type-specific golden markdown first
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
                # Use relative path from testdata (e.g., "docx/file.docx")
                relative_path = f"{subdir_name}/{input_file.name}"
                test_id = relative_path
                pairs.append((relative_path, golden_md, test_id))

    return sorted(pairs, key=lambda x: x[2])


# Generate test parameters
test_file_pairs = find_test_file_pairs()


@pytest.mark.asyncio
async def test_parse_url_returns_ir_and_markdown(base_url, auth_headers, mock_http_server):
    """
    Test POST /file-parser/v1/parse-url endpoint with render_markdown=true.

    This test:
    1. Uses the mock HTTP server to serve test files
    2. Sends a URL to the backend
    3. Verifies the response contains both IR and markdown
    4. Compares markdown with golden reference
    """
    # Find test files
    test_pairs = find_test_file_pairs()

    if not test_pairs:
        pytest.skip("No test file pairs found with golden markdown")

    # Use the first available test file
    filename, golden_md, test_id = test_pairs[0]

    # Construct URL using mock server helper
    file_url = mock_url(filename)

    # Read golden markdown
    golden_markdown = golden_md.read_text(encoding="utf-8")

    # Call API endpoint
    url = f"{base_url}/file-parser/v1/parse-url"
    params = {"render_markdown": "true"}
    request_body = {"url": file_url}

    async with httpx.AsyncClient(timeout=30.0) as client:
        response = await client.post(
            url,
            params=params,
            headers={**auth_headers, "Content-Type": "application/json"},
            json=request_body
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

    # Parse JSON response
    data = response.json()

    # Validate response structure (ParsedDocResponseDto)
    assert "document" in data, f"Response should contain 'document' field for {test_id}"
    assert "markdown" in data, f"Response should contain 'markdown' field for {test_id}"

    # Validate document structure
    document = data["document"]
    assert isinstance(document, dict), f"'document' should be an object for {test_id}"
    assert "meta" in document, f"'document' should contain 'meta' field for {test_id}"
    assert "blocks" in document, f"'document' should contain 'blocks' field for {test_id}"

    blocks = document["blocks"]
    assert isinstance(blocks, list), f"'blocks' should be a list for {test_id}"
    assert len(blocks) > 0, f"'blocks' should not be empty for {test_id}"

    # Validate markdown field
    markdown = data["markdown"]
    assert markdown is not None, f"'markdown' should not be null for {test_id}"
    assert isinstance(markdown, str), f"'markdown' should be a string for {test_id}"
    assert len(markdown) > 0, f"'markdown' should not be empty for {test_id}"

    # Compare with golden reference
    actual_normalized = normalize_markdown(markdown)
    expected_normalized = normalize_markdown(golden_markdown)

    assert actual_normalized == expected_normalized, (
        f"Markdown mismatch for {test_id}. "
        f"First difference at character {_find_first_diff(actual_normalized, expected_normalized)}"
    )


@pytest.mark.asyncio
@pytest.mark.parametrize("relative_path,golden_md,test_id", test_file_pairs, ids=[p[2] for p in test_file_pairs])
async def test_parse_url_all_files(
    base_url, auth_headers, mock_http_server, relative_path, golden_md, test_id
):
    """
    Test POST /file-parser/v1/parse-url for all test files with golden markdown.

    This test is parametrized over all test files with golden references.
    """
    # Construct URL using mock server helper
    file_url = mock_url(relative_path)

    # Read golden markdown
    golden_markdown = golden_md.read_text(encoding="utf-8")

    # Call API endpoint
    url = f"{base_url}/file-parser/v1/parse-url"
    params = {"render_markdown": "true"}
    request_body = {"url": file_url}

    async with httpx.AsyncClient(timeout=30.0) as client:
        response = await client.post(
            url,
            params=params,
            headers={**auth_headers, "Content-Type": "application/json"},
            json=request_body
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

    # Parse JSON response
    data = response.json()

    # Validate markdown field
    markdown = data.get("markdown")
    assert markdown is not None, f"'markdown' should not be null for {test_id}"

    # Compare with golden reference
    actual_normalized = normalize_markdown(markdown)
    expected_normalized = normalize_markdown(golden_markdown)

    assert actual_normalized == expected_normalized, (
        f"Markdown mismatch for {test_id}. "
        f"First difference at character {_find_first_diff(actual_normalized, expected_normalized)}"
    )


@pytest.mark.asyncio
async def test_parse_url_markdown_stream(base_url, auth_headers, mock_http_server):
    """
    Test POST /file-parser/v1/parse-url/markdown endpoint.

    This test:
    1. Uses the mock HTTP server to serve test files
    2. Sends a URL to the backend
    3. Expects text/markdown response
    4. Compares markdown with golden reference
    """
    # Find test files
    test_pairs = find_test_file_pairs()

    if not test_pairs:
        pytest.skip("No test file pairs found with golden markdown")

    # Use the first available test file
    relative_path, golden_md, test_id = test_pairs[0]

    # Construct URL using mock server helper
    file_url = mock_url(relative_path)

    # Read golden markdown
    golden_markdown = golden_md.read_text(encoding="utf-8")

    # Call API endpoint
    url = f"{base_url}/file-parser/v1/parse-url/markdown"
    request_body = {"url": file_url}

    async with httpx.AsyncClient(timeout=30.0) as client:
        response = await client.post(
            url,
            headers={**auth_headers, "Content-Type": "application/json"},
            json=request_body
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

    # Compare with golden reference
    actual_normalized = normalize_markdown(actual_markdown)
    expected_normalized = normalize_markdown(golden_markdown)

    assert actual_normalized == expected_normalized, (
        f"Markdown mismatch for {test_id}. "
        f"First difference at character {_find_first_diff(actual_normalized, expected_normalized)}"
    )


@pytest.mark.asyncio
@pytest.mark.parametrize("relative_path,golden_md,test_id", test_file_pairs, ids=[p[2] for p in test_file_pairs])
async def test_parse_url_markdown_all_files(
    base_url, auth_headers, mock_http_server, relative_path, golden_md, test_id
):
    """
    Test POST /file-parser/v1/parse-url/markdown for all test files with golden markdown.

    This test is parametrized over all test files with golden references.
    """
    # Construct URL using mock server helper
    file_url = mock_url(relative_path)

    # Read golden markdown
    golden_markdown = golden_md.read_text(encoding="utf-8")

    # Call API endpoint
    url = f"{base_url}/file-parser/v1/parse-url/markdown"
    request_body = {"url": file_url}

    async with httpx.AsyncClient(timeout=30.0) as client:
        response = await client.post(
            url,
            headers={**auth_headers, "Content-Type": "application/json"},
            json=request_body
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

    # Get response text
    actual_markdown = response.text

    # Validate markdown is not empty
    assert len(actual_markdown) > 0, f"Markdown should not be empty for {test_id}"

    # Compare with golden reference
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
async def test_parse_url_invalid_url(base_url, auth_headers):
    """
    Test POST /file-parser/v1/parse-url with an invalid URL.

    This test verifies error handling for invalid URLs.
    """
    # Use an invalid/unreachable URL
    invalid_url = "http://invalid-host-that-does-not-exist-12345.com/file.pdf"

    # Call API endpoint
    url = f"{base_url}/file-parser/v1/parse-url"
    request_body = {"url": invalid_url}

    async with httpx.AsyncClient(timeout=30.0) as client:
        response = await client.post(
            url,
            headers={**auth_headers, "Content-Type": "application/json"},
            json=request_body
        )

    # Handle auth requirements
    if response.status_code in (401, 403) and not auth_headers:
        pytest.skip(
            f"Endpoint requires authentication (got {response.status_code}). "
            "Set E2E_AUTH_TOKEN environment variable to run this test."
        )

    # Expect an error response (not 200)
    # The exact error code depends on backend implementation (could be 400, 404, 502, etc.)
    assert response.status_code != 200, (
        "Expected an error response for invalid URL, but got 200"
    )

    # Just verify we got some kind of error response
    assert response.status_code >= 400, (
        f"Expected error status code (>=400) for invalid URL, got {response.status_code}"
    )
