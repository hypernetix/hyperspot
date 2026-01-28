# File Parser - Quickstart

Parses PDF, DOCX, HTML, Markdown, images, and more into structured document blocks.

Full API documentation: <http://127.0.0.1:8087/docs>

## Examples

### List Supported File Types

```bash
curl -s http://127.0.0.1:8087/file-parser/v1/info | python3 -m json.tool
```

**Output:**
```json
{
    "supported_extensions": {
        "plain_text": ["txt", "log", "md"],
        "html": ["html", "htm"],
        "pdf": ["pdf"],
        "docx": ["docx"],
        "image": ["png", "jpg", "jpeg", "webp", "gif"],
        "generic_stub": ["doc", "rtf", "odt", "xls", "xlsx", "ppt", "pptx"]
    }
}
```

### Upload and Parse a File

```bash
echo "Hello, HyperSpot!" > /tmp/test.txt
curl -s -X POST "http://127.0.0.1:8087/file-parser/v1/upload?filename=test.txt" \
  -H "Content-Type: application/octet-stream" \
  --data-binary @/tmp/test.txt | python3 -m json.tool
```

**Output:**
```json
{
    "document": {
        "id": "019bc231-fcfd-7df3-a49c-82174973ec44",
        "title": "test.txt",
        "meta": {
            "source": {"type": "uploaded", "original_name": "test.txt"},
            "content_type": "text/plain"
        },
        "blocks": [
            {
                "type": "paragraph",
                "inlines": [{"type": "text", "text": "Hello, HyperSpot!", "style": {}}]
            }
        ]
    }
}
```

For additional endpoints, see <http://127.0.0.1:8087/docs>.
