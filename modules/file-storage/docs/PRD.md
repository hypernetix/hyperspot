# PRD

## 1. Overview

**Purpose**: FileStorage provides media storage and retrieval for LLM Gateway inputs and outputs.

FileStorage is a dependency service that handles binary content (images, audio, video, documents) for AI workloads. LLM Gateway fetches user-uploaded media before sending to providers and stores provider-generated content (images, audio, video) for delivery to consumers.

The service provides URL-based access to files with metadata queries for validation (size limits, mime types) before processing.

**Target Users**:
- **LLM Gateway** - Primary consumer for media fetch and store operations
- **Platform Services** - Other services requiring file storage

**Key Problems Solved**:
- **Media handling**: Centralized storage for AI input/output media
- **URL-based access**: Consistent URL scheme for file references
- **Metadata queries**: Pre-fetch validation of file size and type

**Success Criteria**:
- All scenarios (S1-S3) implemented and operational
- File fetch latency appropriate for media size
- Storage availability matches platform SLA

**Capabilities**:
- Fetch media by URL
- Store generated content
- Get file metadata
- Support for images, audio, video, documents

## 2. Actors

### 2.1 Human Actors

<!-- No direct human actors for LLM Gateway scope -->

### 2.2 System Actors

#### LLM Gateway

**ID**: `cpt-cf-file-storage-actor-llm-gateway`

**Role**: Fetches input media (images, audio, video, documents) before provider calls and stores generated output media.

## 3. Functional Requirements

#### Fetch Media by URL

- [ ] `p1` - **ID**: `cpt-cf-file-storage-fr-fetch-media-v1`


The system must retrieve file content and metadata by URL for LLM Gateway consumption.

**Actors**: `cpt-cf-file-storage-actor-llm-gateway`

#### Store Generated Content

- [ ] `p1` - **ID**: `cpt-cf-file-storage-fr-store-content-v1`


The system must store generated media (images, audio, video) and return accessible URL.

**Actors**: `cpt-cf-file-storage-actor-llm-gateway`

#### Get Metadata

- [ ] `p1` - **ID**: `cpt-cf-file-storage-fr-get-metadata-v1`


The system must return file metadata (size, mime_type) without fetching full content for validation purposes.

**Actors**: `cpt-cf-file-storage-actor-llm-gateway`

## 4. Use Cases

#### UC-001: Fetch Media by URL

- [ ] `p1` - **ID**: `cpt-cf-file-storage-usecase-fetch-media-v1`

**Actor**: `cpt-cf-file-storage-actor-llm-gateway`

**Preconditions**: File exists at URL.

**Flow**:
1. LLM Gateway sends fetch(url)
2. FileStorage retrieves file content
3. FileStorage returns content + metadata

**Postconditions**: Content returned to Gateway.

**Acceptance criteria**:
- Returns file_not_found if file does not exist
- Returns content with mime_type and size
- Supports streaming for large files

#### UC-002: Store Generated Content

- [ ] `p1` - **ID**: `cpt-cf-file-storage-usecase-store-content-v1`

**Actor**: `cpt-cf-file-storage-actor-llm-gateway`

**Preconditions**: Content available for storage.

**Flow**:
1. LLM Gateway sends store(content, metadata)
2. FileStorage persists content
3. FileStorage returns accessible URL

**Postconditions**: File stored, URL returned.

**Acceptance criteria**:
- URL immediately accessible after store returns
- Metadata stored with file (mime_type, source, timestamps)
- Storage_unavailable error if service down

#### UC-003: Get Metadata

- [ ] `p1` - **ID**: `cpt-cf-file-storage-usecase-get-metadata-v1`

**Actor**: `cpt-cf-file-storage-actor-llm-gateway`

**Preconditions**: File exists at URL.

**Flow**:
1. LLM Gateway sends get_metadata(url)
2. FileStorage returns size and mime_type

**Postconditions**: Metadata returned.

**Acceptance criteria**:
- Returns file_not_found if file does not exist
- Faster than full fetch (no content transfer)
- Used for size limit validation before processing

## 5. Non-functional requirements

#### N/A

- [ ] `p1` - **ID**: `cpt-cf-file-storage-nfr-na`

<!-- NFRs to be defined later -->
