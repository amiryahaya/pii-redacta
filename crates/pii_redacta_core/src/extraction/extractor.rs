//! File extraction implementation
//!
//! Sprint 5: File Processing

use crate::error::{PiiError, Result};

/// Document format types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentFormat {
    PlainText,
    Pdf,
    Docx,
}

/// Extracted document content
#[derive(Debug, Clone)]
pub struct ExtractedDocument {
    pub text: String,
    pub format: DocumentFormat,
}

/// Plain text extractor
pub struct TextExtractor;

impl TextExtractor {
    /// Extract text from plain text content
    pub fn extract(content: &[u8], _mime_type: &str) -> Result<ExtractedDocument> {
        let text = String::from_utf8_lossy(content);
        Ok(ExtractedDocument {
            text: text.to_string(),
            format: DocumentFormat::PlainText,
        })
    }

    /// Detect MIME type from content magic bytes
    pub fn detect_mime(content: &[u8]) -> Option<&'static str> {
        if content.len() < 4 {
            return None;
        }

        // PDF magic: %PDF
        if content.starts_with(b"%PDF") {
            return Some("application/pdf");
        }

        // ZIP magic: PK (used by DOCX, XLSX, etc.)
        if content.starts_with(b"PK\x03\x04") || content.starts_with(b"PK\x05\x06") {
            return Some("application/vnd.openxmlformats");
        }

        // Plain text has no reliable magic bytes
        None
    }
}

/// PDF text extractor (simplified implementation)
pub struct PdfExtractor;

impl PdfExtractor {
    /// Extract text from PDF content
    ///
    /// This is a simplified implementation that looks for text streams.
    /// A production implementation would use a proper PDF parsing library.
    pub fn extract(content: &[u8]) -> Result<ExtractedDocument> {
        // Check for PDF magic bytes
        if !content.starts_with(b"%PDF") {
            return Err(PiiError::ExtractionFailed(
                "Not a valid PDF file".to_string(),
            ));
        }

        // Simple text extraction: look for text between parentheses in streams
        // This is a naive approach for testing purposes
        let content_str = String::from_utf8_lossy(content);
        let mut extracted_text = String::new();

        // Look for text in BT...ET blocks (Begin/End Text)
        for line in content_str.lines() {
            // Extract text between parentheses (simplified)
            if let Some(start) = line.find('(') {
                if let Some(end) = line.rfind(')') {
                    if end > start {
                        let text = &line[start + 1..end];
                        // Skip escaped parentheses and simple escapes
                        let cleaned = text
                            .replace("\\(", "(")
                            .replace("\\)", ")")
                            .replace("\\\\", "\\");
                        if !cleaned.trim().is_empty() {
                            extracted_text.push_str(&cleaned);
                            extracted_text.push(' ');
                        }
                    }
                }
            }
        }

        Ok(ExtractedDocument {
            text: extracted_text.trim().to_string(),
            format: DocumentFormat::Pdf,
        })
    }
}

/// DOCX text extractor (simplified implementation)
pub struct DocxExtractor;

impl DocxExtractor {
    /// Extract text from DOCX content
    ///
    /// This is a simplified implementation.
    /// A production implementation would properly parse the ZIP structure
    /// and extract text from word/document.xml
    pub fn extract(content: &[u8]) -> Result<ExtractedDocument> {
        // Check for ZIP magic bytes (DOCX is a ZIP file)
        if !content.starts_with(b"PK") {
            return Err(PiiError::ExtractionFailed(
                "Not a valid DOCX file".to_string(),
            ));
        }

        // For this simplified implementation, we just extract any readable text
        // A real implementation would:
        // 1. Parse the ZIP structure
        // 2. Extract word/document.xml
        // 3. Parse the XML and extract text between <w:t> tags

        let content_str = String::from_utf8_lossy(content);
        let mut extracted_text = String::new();

        // Extract readable ASCII text
        for chunk in content_str.split(|c: char| c.is_control() && c != '\n' && c != '\t') {
            let trimmed = chunk.trim();
            if trimmed.len() > 3
                && trimmed
                    .chars()
                    .all(|c| c.is_ascii_graphic() || c.is_whitespace())
            {
                extracted_text.push_str(trimmed);
                extracted_text.push(' ');
            }
        }

        Ok(ExtractedDocument {
            text: extracted_text.trim().to_string(),
            format: DocumentFormat::Docx,
        })
    }
}

/// Main extractor that dispatches to format-specific extractors
pub struct Extractor;

impl Extractor {
    /// Extract text from document bytes
    ///
    /// If mime_type is None, attempts to auto-detect from content
    pub fn extract(content: &[u8], mime_type: Option<&str>) -> Result<ExtractedDocument> {
        let mime = match mime_type {
            Some(m) => m,
            None => {
                // Auto-detect from content
                TextExtractor::detect_mime(content).unwrap_or("text/plain")
            }
        };

        match mime {
            "text/plain" => TextExtractor::extract(content, mime),
            "application/pdf" => PdfExtractor::extract(content),
            "application/vnd.openxmlformats"
            | "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
                DocxExtractor::extract(content)
            }
            _ => Err(PiiError::ExtractionFailed(format!(
                "Unsupported MIME type: {}",
                mime
            ))),
        }
    }
}

#[cfg(test)]
#[path = "extractor_test.rs"]
mod tests;
