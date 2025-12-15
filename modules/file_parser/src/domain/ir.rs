use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Intermediate representation of a parsed document
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedDocument {
    pub id: Option<Uuid>,
    pub title: Option<String>,
    pub language: Option<String>, // BCP 47, e.g., "en", "ru"
    pub meta: ParsedMetadata,
    pub blocks: Vec<ParsedBlock>,
}

/// Metadata about the parsed document
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedMetadata {
    pub source: ParsedSource,
    pub original_filename: Option<String>,
    pub content_type: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub modified_at: Option<DateTime<Utc>>,
    pub is_stub: bool,
}

/// Source of the parsed document
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedSource {
    LocalPath(String),
    Uploaded { original_name: String },
    Url(String),
}

/// Inline-level text styling
#[derive(Debug, Clone, PartialEq, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct InlineStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub code: bool,
}

/// Inline-level content elements
#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    Text {
        text: String,
        style: InlineStyle,
    },
    Link {
        text: String,
        target: String,
        style: InlineStyle,
    },
    Code {
        text: String,
        style: InlineStyle,
    },
}

impl Inline {
    /// Create plain text with no styling
    pub fn plain(text: impl Into<String>) -> Self {
        Inline::Text {
            text: text.into(),
            style: InlineStyle::default(),
        }
    }

    /// Create text with custom style
    pub fn styled(text: impl Into<String>, style: InlineStyle) -> Self {
        Inline::Text {
            text: text.into(),
            style,
        }
    }

    /// Create a link
    pub fn link(text: impl Into<String>, target: impl Into<String>) -> Self {
        Inline::Link {
            text: text.into(),
            target: target.into(),
            style: InlineStyle::default(),
        }
    }

    /// Create inline code
    pub fn code(text: impl Into<String>) -> Self {
        Inline::Code {
            text: text.into(),
            style: InlineStyle::default(),
        }
    }
}

/// Structured table representation
#[derive(Debug, Clone, PartialEq)]
pub struct TableBlock {
    pub rows: Vec<TableRow>,
}

/// A single row in a table
#[derive(Debug, Clone, PartialEq)]
pub struct TableRow {
    pub is_header: bool,
    pub cells: Vec<TableCell>,
}

/// A single cell in a table, containing block-level content
#[derive(Debug, Clone, PartialEq)]
pub struct TableCell {
    pub blocks: Vec<ParsedBlock>,
}

/// Block-level elements in the document
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedBlock {
    Heading {
        level: u8, // 1-6
        inlines: Vec<Inline>,
    },
    Paragraph {
        inlines: Vec<Inline>,
    },
    ListItem {
        level: u8, // 0-based nesting level
        ordered: bool,
        blocks: Vec<ParsedBlock>,
    },
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    Table(TableBlock),
    Quote {
        blocks: Vec<ParsedBlock>,
    },
    HorizontalRule,
    Image {
        alt: Option<String>,
        title: Option<String>,
        src: Option<String>,
    },
    PageBreak,
}

/// Builder for constructing `ParsedDocument` in a fluent style
#[must_use]
pub struct DocumentBuilder {
    id: Option<Uuid>,
    title: Option<String>,
    language: Option<String>,
    source: ParsedSource,
    original_filename: Option<String>,
    content_type: Option<String>,
    created_at: Option<DateTime<Utc>>,
    modified_at: Option<DateTime<Utc>>,
    is_stub: bool,
    blocks: Vec<ParsedBlock>,
}

impl DocumentBuilder {
    /// Create a new document builder with the given source
    pub fn new(source: ParsedSource) -> Self {
        Self {
            id: Some(Uuid::now_v7()),
            title: None,
            language: None,
            source,
            original_filename: None,
            content_type: None,
            created_at: None,
            modified_at: None,
            is_stub: false,
            blocks: Vec::new(),
        }
    }

    /// Set the document ID
    pub fn id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    /// Set the document title
    pub fn title<T: Into<String>>(mut self, title: T) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the document language
    pub fn language<T: Into<String>>(mut self, language: T) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Set the original filename
    pub fn original_filename<T: Into<String>>(mut self, name: T) -> Self {
        self.original_filename = Some(name.into());
        self
    }

    /// Set the content type
    pub fn content_type<T: Into<String>>(mut self, content_type: T) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    /// Set the created timestamp
    pub fn created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = Some(created_at);
        self
    }

    /// Set the modified timestamp
    pub fn modified_at(mut self, modified_at: DateTime<Utc>) -> Self {
        self.modified_at = Some(modified_at);
        self
    }

    /// Set whether this is a stub parser output
    pub fn stub(mut self, is_stub: bool) -> Self {
        self.is_stub = is_stub;
        self
    }

    /// Set the document blocks
    pub fn blocks(mut self, blocks: Vec<ParsedBlock>) -> Self {
        self.blocks = blocks;
        self
    }

    /// Build the `ParsedDocument`
    #[must_use]
    pub fn build(self) -> ParsedDocument {
        ParsedDocument {
            id: self.id.or_else(|| Some(Uuid::now_v7())),
            title: self.title,
            language: self.language,
            meta: ParsedMetadata {
                source: self.source,
                original_filename: self.original_filename,
                content_type: self.content_type,
                created_at: self.created_at,
                modified_at: self.modified_at,
                is_stub: self.is_stub,
            },
            blocks: self.blocks,
        }
    }
}
