"""E2E tests for image file parsing via /file-parser/v1/upload endpoint."""
import base64
import httpx
import pytest
from pathlib import Path


def get_image_test_files():
    """
    Find all image test files.

    Returns:
        List of tuples: (image_file_path, expected_mime_type, test_id)
    """
    testdata_dir = Path(__file__).parent.parent.parent / "testdata" / "images"

    if not testdata_dir.exists():
        return []

    files = []
    
    # Map extensions to expected MIME types
    mime_types = {
        ".png": "image/png",
        ".jpg": "image/jpeg",
        ".jpeg": "image/jpeg",
        ".webp": "image/webp",
        ".gif": "image/gif",
    }

    for image_file in testdata_dir.iterdir():
        if not image_file.is_file():
            continue

        ext = image_file.suffix.lower()
        if ext in mime_types:
            test_id = f"images/{image_file.name}"
            files.append((image_file, mime_types[ext], test_id))

    return sorted(files, key=lambda x: x[2])


# Generate test parameters
image_test_files = get_image_test_files()


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "image_file,expected_mime,test_id",
    image_test_files,
    ids=[f[2] for f in image_test_files]
)
async def test_upload_image(base_url, auth_headers, image_file, expected_mime, test_id):
    """
    Test POST /file-parser/v1/upload endpoint with image files.

    This test:
    1. Uploads an image file
    2. Validates the response structure
    3. Verifies MIME type detection
    4. Validates the base64 data URI
    5. Verifies byte-for-byte equality after base64 decode
    """
    # Read image file
    with open(image_file, "rb") as f:
        original_bytes = f.read()

    async with httpx.AsyncClient() as client:
        # Upload image
        response = await client.post(
            f"{base_url}/file-parser/v1/upload",
            params={"filename": image_file.name},
            headers={**auth_headers, "Content-Type": "application/octet-stream"},
            content=original_bytes,
            timeout=30.0,
        )

        assert response.status_code == 200, f"Upload failed: {response.text}"

        data = response.json()

        # Validate response structure
        assert "document" in data, "Response should contain 'document'"
        document = data["document"]

        # Validate metadata
        assert document["meta"]["content_type"] == expected_mime, \
            f"Expected MIME type {expected_mime}, got {document['meta']['content_type']}"
        assert document["meta"]["original_filename"] == image_file.name, \
            f"Expected filename {image_file.name}, got {document['meta']['original_filename']}"

        # Validate blocks structure
        assert len(document["blocks"]) == 1, "Should have exactly one block"
        block = document["blocks"][0]
        
        assert block["type"] == "image", f"Expected image block, got {block['type']}"
        assert "src" in block, "Image block should have 'src' field"
        
        data_uri = block["src"]
        assert data_uri is not None, "Image src should not be null"

        # Validate data URI format
        expected_prefix = f"data:{expected_mime};base64,"
        assert data_uri.startswith(expected_prefix), \
            f"Data URI should start with '{expected_prefix}'"

        # Extract and decode base64
        base64_data = data_uri[len(expected_prefix):]
        decoded_bytes = base64.b64decode(base64_data)

        # Verify byte-for-byte equality
        assert decoded_bytes == original_bytes, \
            f"Decoded bytes should match original file bytes (original: {len(original_bytes)} bytes, decoded: {len(decoded_bytes)} bytes)"


@pytest.mark.asyncio
async def test_upload_image_png_detailed(base_url, auth_headers):
    """
    Detailed test for PNG upload with comprehensive validation.
    """
    testdata_dir = Path(__file__).parent.parent.parent / "testdata" / "images"
    png_file = testdata_dir / "tiny.png"

    if not png_file.exists():
        pytest.skip(f"Test file not found: {png_file}")

    with open(png_file, "rb") as f:
        file_content = f.read()

    async with httpx.AsyncClient() as client:
        response = await client.post(
            f"{base_url}/file-parser/v1/upload",
            params={"filename": "tiny.png"},
            headers={**auth_headers, "Content-Type": "application/octet-stream"},
            content=file_content,
            timeout=30.0,
        )

        assert response.status_code == 200
        data = response.json()

        # Detailed validation
        document = data["document"]
        assert document["meta"]["content_type"] == "image/png"
        assert document["blocks"][0]["type"] == "image"
        
        # Validate optional fields
        assert document["blocks"][0].get("alt") is None or isinstance(document["blocks"][0].get("alt"), str)
        assert document["blocks"][0].get("title") is None or isinstance(document["blocks"][0].get("title"), str)


@pytest.mark.asyncio
async def test_upload_image_without_content_type(base_url, auth_headers):
    """
    Test that image upload works even without explicit content-type (relies on extension).
    """
    testdata_dir = Path(__file__).parent.parent.parent / "testdata" / "images"
    jpg_file = testdata_dir / "tiny.jpg"

    if not jpg_file.exists():
        pytest.skip(f"Test file not found: {jpg_file}")

    with open(jpg_file, "rb") as f:
        file_content = f.read()

    async with httpx.AsyncClient() as client:
        # Upload without explicit content-type (using application/octet-stream)
        response = await client.post(
            f"{base_url}/file-parser/v1/upload",
            params={"filename": "tiny.jpg"},
            headers={**auth_headers, "Content-Type": "application/octet-stream"},
            content=file_content,
            timeout=30.0,
        )

        assert response.status_code == 200
        data = response.json()

        # Should still detect JPEG based on extension
        document = data["document"]
        assert document["meta"]["content_type"] == "image/jpeg"


@pytest.mark.asyncio
async def test_upload_unsupported_image_format(base_url, auth_headers):
    """
    Test that unsupported image formats (e.g., .bmp, .tiff) are rejected.
    """
    # Create a fake BMP file (just some bytes with .bmp extension)
    fake_bmp_content = b"BM\x00\x00\x00\x00\x00\x00\x00\x00"

    async with httpx.AsyncClient() as client:
        response = await client.post(
            f"{base_url}/file-parser/v1/upload",
            params={"filename": "test.bmp"},
            headers={**auth_headers, "Content-Type": "application/octet-stream"},
            content=fake_bmp_content,
            timeout=30.0,
        )

        # Should return an error (unsupported file type)
        assert response.status_code in [400, 415, 422, 500], \
            f"Expected error status for unsupported format, got {response.status_code}"


@pytest.mark.asyncio
async def test_upload_image_all_formats(base_url, auth_headers):
    """
    Test that all supported image formats can be uploaded and parsed.
    """
    testdata_dir = Path(__file__).parent.parent.parent / "testdata" / "images"
    
    formats = [
        ("tiny.png", "image/png"),
        ("tiny.jpg", "image/jpeg"),
        ("tiny.webp", "image/webp"),
        ("tiny.gif", "image/gif"),
    ]

    for filename, expected_mime in formats:
        image_file = testdata_dir / filename
        
        if not image_file.exists():
            pytest.skip(f"Test file not found: {image_file}")
            continue

        with open(image_file, "rb") as f:
            file_content = f.read()

        async with httpx.AsyncClient() as client:
            response = await client.post(
                f"{base_url}/file-parser/v1/upload",
                params={"filename": filename},
                headers={**auth_headers, "Content-Type": "application/octet-stream"},
                content=file_content,
                timeout=30.0,
            )

            assert response.status_code == 200, \
                f"Failed to upload {filename}: {response.text}"

            data = response.json()
            assert data["document"]["meta"]["content_type"] == expected_mime, \
                f"Wrong MIME type for {filename}"
