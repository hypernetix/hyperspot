# File Parser Module

File parsing module for CyberFabric / ModKit.

## Overview

The `cf-file-parser` crate implements the `file-parser` module and registers REST routes.

Parsing backends currently include:

- Plain text
- HTML
- PDF
- DOCX
- Images
- Stub parser (fallback)

## Configuration

```yaml
modules:
  file-parser:
    config:
      max_file_size_mb: 100
      download_timeout_secs: 60
```

## License

Licensed under Apache-2.0.
