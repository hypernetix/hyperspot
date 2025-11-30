pub mod docx_parser;
pub mod html_parser;
pub mod pdf_parser;
pub mod plain_text;
pub mod stub;

pub use docx_parser::DocxParser;
pub use html_parser::HtmlParser;
pub use pdf_parser::PdfParser;
pub use plain_text::PlainTextParser;
pub use stub::StubParser;
