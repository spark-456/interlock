//! Bounded text chunks with source metadata.

use chrono::{DateTime, Utc};

use crate::parse::ParsedDocument;

/// A passage to embed and index.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub text: String,
    pub title: String,
    pub section: Option<String>,
    pub date: DateTime<Utc>,
}

/// Splits sections into overlapping passages without cutting UTF-8 bytes.
pub fn split(document: &ParsedDocument, date: DateTime<Utc>) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    for section in &document.sections {
        let chars: Vec<char> = section.text.chars().collect();
        let mut start = 0;
        while start < chars.len() {
            let end = (start + 1000).min(chars.len());
            let text: String = chars[start..end].iter().collect();
            if !text.trim().is_empty() {
                chunks.push(Chunk { text, title: document.title.clone(), section: section.heading.clone(), date });
            }
            if end == chars.len() {
                break;
            }
            start = end - 100;
        }
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::Section;

    #[test]
    fn k01_chunks_keep_section_and_unicode() {
        let date = Utc::now();
        let doc = ParsedDocument {
            title: "Guide".into(),
            sections: vec![Section { heading: Some("Refunds".into()), text: "é".repeat(1100) }],
        };
        let chunks = split(&doc, date);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[1].section.as_deref(), Some("Refunds"));
        assert!(chunks[1].text.starts_with('é'));
    }
}
