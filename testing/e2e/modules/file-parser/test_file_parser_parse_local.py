"""E2E tests for /file-parser/v1/parse-local and /file-parser/v1/parse-local/markdown endpoints."""
import httpx
import pytest
from pathlib import Path
import os


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


def find_test_file_pairs(local_files_root):
    """
    Find all (file_path, golden_md_file) pairs for testing.

    Args:
        local_files_root: Root directory where test files are located

    Returns:
        List of tuples: (file_path_str, golden_md_path, test_id)
    """
    md_dir = Path(__file__).parent / "testdata" / "md"

    if not local_files_root.exists():
        return []

    pairs = []

    # Scan for input files in subdirectories
    for subdir_name in ["docx", "pdf"]:
        subdir = local_files_root / subdir_name
        if not subdir.exists():
            continue

        for input_file in subdir.iterdir():
            if not input_file.is_file():
                continue

            # Skip non-document files (reference text files)
            if input_file.suffix.lower() in [".txt", ".text", ".md"]:
                continue

            # Check if corresponding golden markdown exists
            golden_md = md_dir / f"{input_file.stem}.md"
            if golden_md.exists():
                # Use absolute path for file_path to ensure backend can access it
                file_path_str = str(input_file.resolve())
                test_id = f"{subdir_name}/{input_file.name}"
                pairs.append((file_path_str, golden_md, test_id))

    return sorted(pairs, key=lambda x: x[2])


@pytest.fixture
def skip_if_local_files_unavailable(local_files_root):
    """Skip tests if local files root is not properly configured."""
    if not local_files_root.exists():
        pytest.skip(
            f"Local files root not found at {local_files_root}. "
            "Set E2E_LOCAL_FILES_ROOT to enable parse-local tests."
        )

    # Also check if there are any test files
    has_files = False
    for subdir_name in ["docx", "pdf"]:
        subdir = local_files_root / subdir_name
        if subdir.exists() and any(
            f.is_file() and f.suffix.lower() in [".pdf", ".docx"]
            for f in subdir.iterdir()
        ):
            has_files = True
            break

    if not has_files:
        pytest.skip(
            f"No test files found in {local_files_root}. "
            "Ensure test files are accessible to the backend."
        )


@pytest.mark.asyncio
async def test_parse_local_returns_ir_and_markdown(
    base_url, auth_headers, local_files_root, skip_if_local_files_unavailable
):
    """
    Test POST /file-parser/v1/parse-local endpoint with render_markdown=true.

    This test:
    1. Sends a local file path to the backend
    2. Verifies the response contains both IR and markdown
    3. Compares markdown with golden reference if available
    """
    # Find test files
    test_pairs = find_test_file_pairs(local_files_root)

    if not test_pairs:
        pytest.skip("No test file pairs found with golden markdown")

    # Use the first available test file
    file_path, golden_md, test_id = test_pairs[0]

    # Read golden markdown
    golden_markdown = golden_md.read_text(encoding="utf-8")

    # Call API endpoint
    url = f"{base_url}/file-parser/v1/parse-local"
    params = {"render_markdown": "true"}
    request_body = {"file_path": file_path}

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

    # If file not found or not accessible, skip with a clear message
    if response.status_code == 404:
        pytest.skip(
            f"Backend cannot access file at {file_path}. "
            "Ensure backend has access to test files."
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
    # Markdown could be null or a string depending on implementation
    if markdown is not None:
        assert isinstance(markdown, str), f"'markdown' should be a string for {test_id}"
        assert len(markdown) > 0, f"'markdown' should not be empty for {test_id}"

        # Compare with golden reference
        actual_normalized = normalize_markdown(markdown)
        expected_normalized = normalize_markdown(golden_markdown)

        assert actual_normalized == expected_normalized, (
            f"Markdown mismatch for {test_id}. "
            f"First difference at character {_find_first_diff(actual_normalized, expected_normalized)}"
        )


def pytest_generate_tests(metafunc):
    """Dynamically generate test parameters for parse_local tests."""
    if "file_path" in metafunc.fixturenames:
        # Get local_files_root from config
        local_files_root_env = os.getenv("E2E_LOCAL_FILES_ROOT")
        if local_files_root_env:
            local_files_root = Path(local_files_root_env).resolve()
        else:
            local_files_root = Path(__file__).parent.parent.parent / "testdata"
            local_files_root = local_files_root.resolve()

        # Find test pairs
        test_pairs = find_test_file_pairs(local_files_root)

        if test_pairs:
            file_paths = [p[0] for p in test_pairs]
            golden_mds = [p[1] for p in test_pairs]
            test_ids = [p[2] for p in test_pairs]

            metafunc.parametrize(
                "file_path,golden_md,test_id",
                list(zip(file_paths, golden_mds, test_ids)),
                ids=test_ids
            )
        else:
            # Mark the test to be skipped if no parameters are found
            metafunc.parametrize(
                "file_path,golden_md,test_id",
                [],
                ids=[]
            )


@pytest.mark.asyncio
async def test_parse_local_all_files(
    base_url, auth_headers, local_files_root, file_path, golden_md, test_id
):
    """
    Test POST /file-parser/v1/parse-local for all test files with golden markdown.

    This test is parametrized but may not run if local files aren't configured.
    """
    # This test will be skipped if no parameters are provided
    # It's here as a template for future use when local file access is fully configured
    pass


@pytest.mark.asyncio
async def test_parse_local_markdown_stream(
    base_url, auth_headers, local_files_root, skip_if_local_files_unavailable
):
    """
    Test POST /file-parser/v1/parse-local/markdown endpoint.

    This test:
    1. Sends a local file path to the backend
    2. Expects text/markdown response
    3. Compares markdown with golden reference if available
    """
    # Find test files
    test_pairs = find_test_file_pairs(local_files_root)

    if not test_pairs:
        pytest.skip("No test file pairs found with golden markdown")

    # Use the first available test file
    file_path, golden_md, test_id = test_pairs[0]

    # Read golden markdown
    golden_markdown = golden_md.read_text(encoding="utf-8")

    # Call API endpoint
    url = f"{base_url}/file-parser/v1/parse-local/markdown"
    request_body = {"file_path": file_path}

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

    # If file not found or not accessible, skip with a clear message
    if response.status_code == 404:
        pytest.skip(
            f"Backend cannot access file at {file_path}. "
            "Ensure backend has access to test files."
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


def _find_first_diff(s1, s2):
    """Find the first character position where two strings differ."""
    for i, (c1, c2) in enumerate(zip(s1, s2)):
        if c1 != c2:
            return i
    return min(len(s1), len(s2))
