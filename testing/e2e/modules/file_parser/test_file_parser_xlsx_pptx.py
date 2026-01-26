"""E2E tests for file_parser XLSX and PPTX support."""
import httpx
import pytest
from pathlib import Path


def get_testdata_dir():
    """Get the testdata directory path."""
    return Path(__file__).parent.parent.parent / "testdata"


@pytest.mark.asyncio
async def test_xlsx_upload_simple(base_url, auth_headers):
    """Test uploading a simple XLSX file."""
    if not auth_headers:
        pytest.skip("Auth not configured. Set E2E_AUTH_TOKEN to run this test.")

    xlsx_file = get_testdata_dir() / "xlsx" / "simple_data.xlsx"

    if not xlsx_file.exists():
        pytest.skip(f"Test file not found: {xlsx_file}")

    with open(xlsx_file, "rb") as f:
        file_content = f.read()

    async with httpx.AsyncClient(timeout=30.0) as client:
        response = await client.post(
            f"{base_url}/file-parser/v1/upload",
            params={"render_markdown": "true", "filename": "simple_data.xlsx"},
            headers={**auth_headers, "Content-Type": "application/octet-stream"},
            content=file_content
        )

    assert response.status_code == 200, f"Upload failed: {response.text}"

    data = response.json()
    assert "document" in data
    assert "markdown" in data

    document = data["document"]
    assert document["meta"]["original_filename"] == "simple_data.xlsx"
    assert "spreadsheetml" in document["meta"]["content_type"]

    # Should have blocks (heading for sheet + table)
    assert len(document["blocks"]) > 0


@pytest.mark.asyncio
async def test_xlsx_upload_multisheet(base_url, auth_headers):
    """Test uploading an XLSX file with multiple sheets."""
    if not auth_headers:
        pytest.skip("Auth not configured. Set E2E_AUTH_TOKEN to run this test.")

    xlsx_file = get_testdata_dir() / "xlsx" / "multi_sheet.xlsx"

    if not xlsx_file.exists():
        pytest.skip(f"Test file not found: {xlsx_file}")

    with open(xlsx_file, "rb") as f:
        file_content = f.read()

    async with httpx.AsyncClient(timeout=30.0) as client:
        response = await client.post(
            f"{base_url}/file-parser/v1/upload",
            params={"render_markdown": "true", "filename": "multi_sheet.xlsx"},
            headers={**auth_headers, "Content-Type": "application/octet-stream"},
            content=file_content
        )

    assert response.status_code == 200, f"Upload failed: {response.text}"

    data = response.json()
    document = data["document"]

    # Count heading blocks (one per sheet)
    heading_count = sum(1 for b in document["blocks"] if b.get("type") == "heading")
    assert heading_count >= 2, f"Expected at least 2 sheet headings, got {heading_count}"


@pytest.mark.asyncio
async def test_pptx_upload_simple(base_url, auth_headers):
    """Test uploading a simple PPTX file."""
    if not auth_headers:
        pytest.skip("Auth not configured. Set E2E_AUTH_TOKEN to run this test.")

    pptx_file = get_testdata_dir() / "pptx" / "simple_presentation.pptx"

    if not pptx_file.exists():
        pytest.skip(f"Test file not found: {pptx_file}")

    with open(pptx_file, "rb") as f:
        file_content = f.read()

    async with httpx.AsyncClient(timeout=30.0) as client:
        response = await client.post(
            f"{base_url}/file-parser/v1/upload",
            params={"render_markdown": "true", "filename": "simple_presentation.pptx"},
            headers={**auth_headers, "Content-Type": "application/octet-stream"},
            content=file_content
        )

    assert response.status_code == 200, f"Upload failed: {response.text}"

    data = response.json()
    assert "document" in data
    assert "markdown" in data

    document = data["document"]
    assert document["meta"]["original_filename"] == "simple_presentation.pptx"
    assert "presentationml" in document["meta"]["content_type"]

    # Should have blocks (heading for slide + content)
    assert len(document["blocks"]) > 0


@pytest.mark.asyncio
async def test_pptx_upload_multislide(base_url, auth_headers):
    """Test uploading a PPTX file with multiple slides."""
    if not auth_headers:
        pytest.skip("Auth not configured. Set E2E_AUTH_TOKEN to run this test.")

    pptx_file = get_testdata_dir() / "pptx" / "multi_slide.pptx"

    if not pptx_file.exists():
        pytest.skip(f"Test file not found: {pptx_file}")

    with open(pptx_file, "rb") as f:
        file_content = f.read()

    async with httpx.AsyncClient(timeout=30.0) as client:
        response = await client.post(
            f"{base_url}/file-parser/v1/upload",
            params={"render_markdown": "true", "filename": "multi_slide.pptx"},
            headers={**auth_headers, "Content-Type": "application/octet-stream"},
            content=file_content
        )

    assert response.status_code == 200, f"Upload failed: {response.text}"

    data = response.json()
    document = data["document"]

    # Count heading blocks (one per slide with "Slide N" format)
    slide_headings = [
        b for b in document["blocks"]
        if b.get("type") == "heading" and b.get("level") == 2
    ]
    assert len(slide_headings) >= 2, f"Expected at least 2 slide headings, got {len(slide_headings)}"


@pytest.mark.asyncio
async def test_xlsx_info_endpoint(base_url, auth_headers):
    """Test that XLSX extension is listed in supported formats."""
    async with httpx.AsyncClient(timeout=30.0) as client:
        response = await client.get(
            f"{base_url}/file-parser/v1/info",
            headers=auth_headers,
        )

    if response.status_code in (401, 403) and not auth_headers:
        pytest.skip(
            f"Endpoint requires authentication (got {response.status_code}). "
            "Set E2E_AUTH_TOKEN environment variable to run this test."
        )

    assert response.status_code == 200
    data = response.json()

    # Check that xlsx extensions are supported
    supported_extensions = data.get("supported_extensions", {})
    assert "xlsx" in supported_extensions, "XLSX should be a supported extension"
    
    # Verify the parser supports multiple Excel formats
    xlsx_extensions = supported_extensions["xlsx"]
    assert "xlsx" in xlsx_extensions, "Should support .xlsx"
    assert "xls" in xlsx_extensions, "Should support .xls"
    assert "xlsm" in xlsx_extensions, "Should support .xlsm"
    assert "xlsb" in xlsx_extensions, "Should support .xlsb"


@pytest.mark.asyncio
async def test_pptx_info_endpoint(base_url, auth_headers):
    """Test that PPTX extension is listed in supported formats."""
    async with httpx.AsyncClient(timeout=30.0) as client:
        response = await client.get(
            f"{base_url}/file-parser/v1/info",
            headers=auth_headers,
        )

    if response.status_code in (401, 403) and not auth_headers:
        pytest.skip(
            f"Endpoint requires authentication (got {response.status_code}). "
            "Set E2E_AUTH_TOKEN environment variable to run this test."
        )

    assert response.status_code == 200
    data = response.json()

    # Check that pptx extensions are supported
    supported_extensions = data.get("supported_extensions", {})
    assert "pptx" in supported_extensions, "PPTX should be a supported extension"
    
    # Verify the parser supports PPTX
    pptx_extensions = supported_extensions["pptx"]
    assert "pptx" in pptx_extensions, "Should support .pptx"
