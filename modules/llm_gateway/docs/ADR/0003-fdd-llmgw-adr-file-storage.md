# ADR-0003: FileStorage for Media Handling

**Date**: 2026-01-29

**Status**: Accepted

**ID**: `fdd-llmgw-adr-file-storage`

## Context and Problem Statement

LLM Gateway handles multimodal content: images, audio, video, documents. How should media be passed between consumers and providers?

## Decision Drivers

* API request size limits — base64-encoded media bloats requests
* Provider URL expiration — provider-generated URLs may expire
* Unified format — different providers return media differently
* Access control — media should respect tenant permissions

## Considered Options

* Inline base64 data in API requests/responses
* Direct provider URLs (pass-through)
* FileStorage URLs for all media

## Decision Outcome

Chosen option: "FileStorage URLs", because it keeps API requests small, provides persistent URLs, and enables access control.

### Consequences

* Good, because API requests stay small (URLs instead of binary data)
* Good, because generated media persists beyond provider URL expiration
* Good, because unified URL format regardless of provider
* Good, because FileStorage handles access control per tenant
* Bad, because adds FileStorage as required dependency
* Bad, because adds latency for media fetch/store operations

## Related Design Elements

**Requirements**:
* `fdd-llmgw-fr-vision-v1` - Fetches images from FileStorage
* `fdd-llmgw-fr-image-generation-v1` - Stores generated images
* `fdd-llmgw-fr-speech-to-text-v1` - Fetches audio from FileStorage
* `fdd-llmgw-fr-text-to-speech-v1` - Stores generated audio
* `fdd-llmgw-fr-video-understanding-v1` - Fetches video from FileStorage
* `fdd-llmgw-fr-video-generation-v1` - Stores generated video
* `fdd-llmgw-fr-document-understanding-v1` - Fetches documents from FileStorage
