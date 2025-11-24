use async_trait::async_trait;
use std::path::Path;

use crate::domain::error::DomainError;
use crate::domain::ir::{DocumentBuilder, Inline, ParsedBlock, ParsedSource};
use crate::domain::parser::FileParserBackend;

/// HTML parser that converts HTML to structured blocks
pub struct HtmlParser;

impl HtmlParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HtmlParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FileParserBackend for HtmlParser {
    fn id(&self) -> &'static str {
        "html"
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        &["html", "htm"]
    }

    async fn parse_local_path(
        &self,
        path: &Path,
    ) -> Result<crate::domain::ir::ParsedDocument, DomainError> {
        let content = tokio::fs::read(path)
            .await
            .map_err(|e| DomainError::io_error(format!("Failed to read file: {}", e)))?;

        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());
        let (blocks, title) =
            tokio::task::spawn_blocking(move || parse_html_bytes(&content, filename.as_deref()))
                .await
                .map_err(|e| DomainError::parse_error(format!("Task join error: {}", e)))??;

        let mut builder = DocumentBuilder::new(ParsedSource::LocalPath(path.display().to_string()))
            .content_type("text/html")
            .blocks(blocks);

        if let Some(t) = title {
            builder = builder.title(t);
        }

        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            builder = builder.original_filename(filename);
        }

        Ok(builder.build())
    }

    async fn parse_bytes(
        &self,
        filename_hint: Option<&str>,
        _content_type: Option<&str>,
        bytes: bytes::Bytes,
    ) -> Result<crate::domain::ir::ParsedDocument, DomainError> {
        let filename_owned = filename_hint.map(|s| s.to_string());
        let (blocks, title) = tokio::task::spawn_blocking(move || {
            parse_html_bytes(&bytes, filename_owned.as_deref())
        })
        .await
        .map_err(|e| DomainError::parse_error(format!("Task join error: {}", e)))??;

        let source = ParsedSource::Uploaded {
            original_name: filename_hint.unwrap_or("unknown.html").to_string(),
        };

        let mut builder = DocumentBuilder::new(source)
            .content_type("text/html")
            .blocks(blocks);

        if let Some(t) = title {
            builder = builder.title(t);
        }

        if let Some(filename) = filename_hint {
            builder = builder.original_filename(filename);
        }

        Ok(builder.build())
    }
}

fn parse_html_bytes(
    bytes: &[u8],
    filename: Option<&str>,
) -> Result<(Vec<ParsedBlock>, Option<String>), DomainError> {
    let html_str = String::from_utf8_lossy(bytes);

    let dom = tl::parse(&html_str, tl::ParserOptions::default())
        .map_err(|e| DomainError::parse_error(format!("Failed to parse HTML: {}", e)))?;

    let parser = dom.parser();
    let mut blocks = Vec::new();
    let mut title = None;

    // Try to extract title from <title> tag
    if let Some(title_node) = dom.query_selector("title").and_then(|mut iter| iter.next()) {
        if let Some(node) = title_node.get(parser) {
            let text = node.inner_text(parser);
            title = Some(text.to_string());
        }
    }

    // Fall back to filename if no title found
    if title.is_none() {
        title = filename.map(|s| s.to_string());
    }

    // Extract body content
    if let Some(body) = dom.query_selector("body").and_then(|mut iter| iter.next()) {
        if let Some(tag) = body.get(parser).and_then(|n| n.as_tag()) {
            extract_blocks_from_node(tag, parser, &mut blocks, 0);
        }
    } else {
        // Fallback: process all top-level nodes
        for node in dom.children() {
            if let Some(tag) = node.get(parser).and_then(|n| n.as_tag()) {
                extract_blocks_from_node(tag, parser, &mut blocks, 0);
            }
        }
    }

    // Fallback: if no blocks extracted, treat as plain text
    if blocks.is_empty() {
        let text = dom.outer_html().trim().to_string();
        if !text.is_empty() {
            blocks.push(ParsedBlock::Paragraph {
                inlines: vec![Inline::plain(text)],
            });
        }
    }

    Ok((blocks, title))
}

fn extract_blocks_from_node(
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    blocks: &mut Vec<ParsedBlock>,
    list_level: u8,
) {
    let tag_name = tag.name().as_utf8_str();

    match tag_name.as_ref() {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let level = tag_name
                .chars()
                .nth(1)
                .and_then(|c| c.to_digit(10))
                .unwrap_or(1) as u8;
            let text = tag.inner_text(parser).trim().to_string();
            if !text.is_empty() {
                // TODO: Parse inline styles from HTML
                blocks.push(ParsedBlock::Heading {
                    level,
                    inlines: vec![Inline::plain(text)],
                });
            }
        }
        "p" => {
            let text = tag.inner_text(parser).trim().to_string();
            if !text.is_empty() {
                // TODO: Parse inline styles from HTML
                blocks.push(ParsedBlock::Paragraph {
                    inlines: vec![Inline::plain(text)],
                });
            }
        }
        "li" => {
            let text = tag.inner_text(parser).trim().to_string();
            if !text.is_empty() {
                // TODO: detect from parent <ol> vs <ul>, parse nested content
                blocks.push(ParsedBlock::ListItem {
                    level: list_level,
                    ordered: false,
                    blocks: vec![ParsedBlock::Paragraph {
                        inlines: vec![Inline::plain(text)],
                    }],
                });
            }
        }
        "ul" | "ol" => {
            let ordered = tag_name.as_ref() == "ol";
            // Process children with increased list level
            for child in tag.children().top().iter() {
                if let Some(child_tag) = child.get(parser).and_then(|n| n.as_tag()) {
                    if child_tag.name().as_utf8_str() == "li" {
                        let text = child_tag.inner_text(parser).trim().to_string();
                        if !text.is_empty() {
                            blocks.push(ParsedBlock::ListItem {
                                level: list_level,
                                ordered,
                                blocks: vec![ParsedBlock::Paragraph {
                                    inlines: vec![Inline::plain(text)],
                                }],
                            });
                        }
                    }
                }
            }
        }
        "pre" | "code" => {
            let code = tag.inner_text(parser).to_string();
            if !code.is_empty() {
                blocks.push(ParsedBlock::CodeBlock {
                    language: None,
                    code,
                });
            }
        }
        "blockquote" => {
            let text = tag.inner_text(parser).trim().to_string();
            if !text.is_empty() {
                // TODO: Parse nested blocks within blockquote
                blocks.push(ParsedBlock::Quote {
                    blocks: vec![ParsedBlock::Paragraph {
                        inlines: vec![Inline::plain(text)],
                    }],
                });
            }
        }
        "hr" => {
            blocks.push(ParsedBlock::HorizontalRule);
        }
        "img" => {
            let alt = tag
                .attributes()
                .get("alt")
                .flatten()
                .map(|b| b.as_utf8_str().to_string());
            let src = tag
                .attributes()
                .get("src")
                .flatten()
                .map(|b| b.as_utf8_str().to_string());
            let title = tag
                .attributes()
                .get("title")
                .flatten()
                .map(|b| b.as_utf8_str().to_string());
            blocks.push(ParsedBlock::Image { alt, title, src });
        }
        _ => {
            // Recurse into children for other tags
            for child in tag.children().top().iter() {
                if let Some(child_tag) = child.get(parser).and_then(|n| n.as_tag()) {
                    extract_blocks_from_node(child_tag, parser, blocks, list_level);
                }
            }
        }
    }
}
