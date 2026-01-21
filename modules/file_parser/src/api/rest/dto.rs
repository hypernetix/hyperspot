use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

/// REST DTO for file parser info response
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(request, response)]
pub struct FileParserInfoDto {
    pub supported_extensions: HashMap<String, Vec<String>>,
}

/// REST DTO for parse local file request
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(request)]
pub struct ParseLocalFileRequest {
    pub file_path: String,
}

/// REST DTO for parse URL request
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(request)]
pub struct ParseUrlRequest {
    pub url: String,
}

/// Query parameters for file upload endpoint
#[derive(Debug, Deserialize)]
pub struct UploadQuery {
    #[serde(default)]
    pub render_markdown: Option<bool>,
    pub filename: Option<String>,
}

/// REST DTO for parsed document metadata
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ParsedDocMetadataDto {
    pub source: ParsedDocSourceDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<OffsetDateTime>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub is_stub: bool,
}

/// REST DTO for document source
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParsedDocSourceDto {
    LocalPath { path: String },
    Uploaded { original_name: String },
    Url { url: String },
}

/// REST DTO for inline text styling
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct InlineStyleDto {
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub bold: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub italic: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub underline: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub strike: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub code: bool,
}

/// REST DTO for inline content
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InlineDto {
    Text {
        text: String,
        style: InlineStyleDto,
    },
    Link {
        text: String,
        target: String,
        style: InlineStyleDto,
    },
    Code {
        text: String,
        style: InlineStyleDto,
    },
}

/// REST DTO for table cell
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TableCellDto {
    #[schema(no_recursion)]
    pub blocks: Vec<ParsedBlockDto>,
}

/// REST DTO for table row
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TableRowDto {
    pub is_header: bool,
    pub cells: Vec<TableCellDto>,
}

/// REST DTO for table block
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TableBlockDto {
    pub rows: Vec<TableRowDto>,
}

/// REST DTO for parsed block
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParsedBlockDto {
    Heading {
        level: u8,
        inlines: Vec<InlineDto>,
    },
    Paragraph {
        inlines: Vec<InlineDto>,
    },
    ListItem {
        level: u8,
        ordered: bool,
        #[schema(no_recursion)]
        blocks: Vec<ParsedBlockDto>,
    },
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    Table {
        #[schema(no_recursion)]
        table: TableBlockDto,
    },
    Quote {
        #[schema(no_recursion)]
        blocks: Vec<ParsedBlockDto>,
    },
    HorizontalRule,
    Image {
        alt: Option<String>,
        title: Option<String>,
        src: Option<String>,
    },
    PageBreak,
}

/// REST DTO for parsed document (IR)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ParsedDocumentDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub meta: ParsedDocMetadataDto,
    pub blocks: Vec<ParsedBlockDto>,
}

/// REST DTO for file parse response (with optional markdown)
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub struct ParsedDocResponseDto {
    /// The parsed document in intermediate representation
    pub document: ParsedDocumentDto,
    /// Rendered markdown (only present when `render_markdown=true`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<String>,
}
