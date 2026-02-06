# Transfer-Encoding validation

## Scenario: unsupported transfer encoding

```http
POST /api/oagw/v1/proxy/httpbin.org/post HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json
Transfer-Encoding: gzip

{"a":1}
```

## Expected response

- `400 Bad Request`
- `Content-Type: application/problem+json`
- `X-OAGW-Error-Source: gateway`
- `detail` mentions unsupported transfer encoding.

## Scenario: chunked supported

```http
POST /api/oagw/v1/proxy/httpbin.org/post HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json
Transfer-Encoding: chunked

<chunked-body>
```

Expected:
- Request is accepted (if upstream supports) and forwarded.
