//! File extraction module
//!
//! Provides text extraction from various document formats.

pub mod extractor;

pub use extractor::{DocxExtractor, Extractor, PdfExtractor, TextExtractor};
