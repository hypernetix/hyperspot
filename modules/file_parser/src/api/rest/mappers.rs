use crate::api::rest::{
    FileParserInfoDto, InlineDto, InlineStyleDto, ParsedBlockDto, ParsedDocMetadataDto,
    ParsedDocSourceDto, ParsedDocumentDto, TableBlockDto, TableCellDto, TableRowDto,
};
use crate::domain::{ir, FileParserInfo};

// Conversion implementations
impl From<FileParserInfo> for FileParserInfoDto {
    fn from(info: FileParserInfo) -> Self {
        Self {
            supported_extensions: info.supported_extensions,
        }
    }
}

impl From<ir::ParsedDocument> for ParsedDocumentDto {
    fn from(doc: ir::ParsedDocument) -> Self {
        Self {
            id: doc.id,
            title: doc.title,
            language: doc.language,
            meta: doc.meta.into(),
            blocks: doc.blocks.into_iter().map(|b| b.into()).collect(),
        }
    }
}

impl From<ir::ParsedMetadata> for ParsedDocMetadataDto {
    fn from(meta: ir::ParsedMetadata) -> Self {
        Self {
            source: meta.source.into(),
            original_filename: meta.original_filename,
            content_type: meta.content_type,
            created_at: meta.created_at,
            modified_at: meta.modified_at,
            is_stub: meta.is_stub,
        }
    }
}

impl From<ir::ParsedSource> for ParsedDocSourceDto {
    fn from(source: ir::ParsedSource) -> Self {
        match source {
            ir::ParsedSource::LocalPath(path) => ParsedDocSourceDto::LocalPath { path },
            ir::ParsedSource::Uploaded { original_name } => {
                ParsedDocSourceDto::Uploaded { original_name }
            }
            ir::ParsedSource::Url(url) => ParsedDocSourceDto::Url { url },
        }
    }
}

impl From<ir::InlineStyle> for InlineStyleDto {
    fn from(style: ir::InlineStyle) -> Self {
        InlineStyleDto {
            bold: style.bold,
            italic: style.italic,
            underline: style.underline,
            strike: style.strike,
            code: style.code,
        }
    }
}

impl From<ir::Inline> for InlineDto {
    fn from(inline: ir::Inline) -> Self {
        match inline {
            ir::Inline::Text { text, style } => InlineDto::Text {
                text,
                style: style.into(),
            },
            ir::Inline::Link {
                text,
                target,
                style,
            } => InlineDto::Link {
                text,
                target,
                style: style.into(),
            },
            ir::Inline::Code { text, style } => InlineDto::Code {
                text,
                style: style.into(),
            },
        }
    }
}

impl From<ir::TableCell> for TableCellDto {
    fn from(cell: ir::TableCell) -> Self {
        TableCellDto {
            blocks: cell.blocks.into_iter().map(|b| b.into()).collect(),
        }
    }
}

impl From<ir::TableRow> for TableRowDto {
    fn from(row: ir::TableRow) -> Self {
        TableRowDto {
            is_header: row.is_header,
            cells: row.cells.into_iter().map(|c| c.into()).collect(),
        }
    }
}

impl From<ir::TableBlock> for TableBlockDto {
    fn from(table: ir::TableBlock) -> Self {
        TableBlockDto {
            rows: table.rows.into_iter().map(|r| r.into()).collect(),
        }
    }
}

impl From<ir::ParsedBlock> for ParsedBlockDto {
    fn from(block: ir::ParsedBlock) -> Self {
        match block {
            ir::ParsedBlock::Heading { level, inlines } => ParsedBlockDto::Heading {
                level,
                inlines: inlines.into_iter().map(|i| i.into()).collect(),
            },
            ir::ParsedBlock::Paragraph { inlines } => ParsedBlockDto::Paragraph {
                inlines: inlines.into_iter().map(|i| i.into()).collect(),
            },
            ir::ParsedBlock::ListItem {
                level,
                ordered,
                blocks,
            } => ParsedBlockDto::ListItem {
                level,
                ordered,
                blocks: blocks.into_iter().map(|b| b.into()).collect(),
            },
            ir::ParsedBlock::CodeBlock { language, code } => {
                ParsedBlockDto::CodeBlock { language, code }
            }
            ir::ParsedBlock::Table(table) => ParsedBlockDto::Table {
                table: table.into(),
            },
            ir::ParsedBlock::Quote { blocks } => ParsedBlockDto::Quote {
                blocks: blocks.into_iter().map(|b| b.into()).collect(),
            },
            ir::ParsedBlock::HorizontalRule => ParsedBlockDto::HorizontalRule,
            ir::ParsedBlock::Image { alt, title, src } => ParsedBlockDto::Image { alt, title, src },
            ir::ParsedBlock::PageBreak => ParsedBlockDto::PageBreak,
        }
    }
}
