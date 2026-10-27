//! Offline document parsing for knowledge ingestion.

use std::io::{Cursor, Read};

use il_sdk::services::knowledge::KnowledgeError;
use quick_xml::{Reader, events::Event};
use scraper::{Html, Selector};

/// One text section with its heading.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub heading: Option<String>,
    pub text: String,
}

/// A document ready for chunking.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedDocument {
    pub title: String,
    pub sections: Vec<Section>,
}

/// Parses supported file formats without network access.
pub fn parse(name: &str, mime: &str, bytes: &[u8]) -> Result<ParsedDocument, KnowledgeError> {
    if bytes.len() > 10 * 1024 * 1024 {
        return Err(KnowledgeError::Ingest("document exceeds 10 MB".into()));
    }
    let suffix = name.rsplit('.').next().unwrap_or_default().to_ascii_lowercase();
    let text = match (mime, suffix.as_str()) {
        ("application/pdf", _) | (_, "pdf") => pdf_extract::extract_text_from_mem(bytes)
            .map_err(|e| KnowledgeError::Ingest(format!("PDF text extraction failed: {e}")))?,
        ("application/vnd.openxmlformats-officedocument.wordprocessingml.document", _) | (_, "docx") => docx(bytes)?,
        ("text/html", _) | (_, "html" | "htm") => return html(name, bytes),
        ("text/csv", _) | (_, "csv") => csv_text(bytes)?,
        ("text/markdown", _) | (_, "md" | "markdown") => {
            return Ok(markdown(name, std::str::from_utf8(bytes).map_err(|e| KnowledgeError::Ingest(e.to_string()))?));
        }
        ("text/plain", _) | (_, "txt") => {
            std::str::from_utf8(bytes).map_err(|e| KnowledgeError::Ingest(e.to_string()))?.to_owned()
        }
        _ => return Err(KnowledgeError::Ingest(format!("unsupported document type: {mime}"))),
    };
    Ok(ParsedDocument { title: name.into(), sections: vec![Section { heading: None, text }] })
}

fn docx(bytes: &[u8]) -> Result<String, KnowledgeError> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| KnowledgeError::Ingest(format!("invalid DOCX: {e}")))?;
    let document = archive
        .by_name("word/document.xml")
        .map_err(|e| KnowledgeError::Ingest(format!("missing DOCX document: {e}")))?;
    let mut xml = String::new();
    document.take(20 * 1024 * 1024 + 1).read_to_string(&mut xml).map_err(|e| KnowledgeError::Ingest(e.to_string()))?;
    if xml.len() > 20 * 1024 * 1024 {
        return Err(KnowledgeError::Ingest("DOCX text exceeds 20 MB".into()));
    }
    let mut reader = Reader::from_str(&xml);
    let mut text = String::new();
    loop {
        match reader.read_event() {
            Ok(Event::Text(value)) => {
                text.push_str(&value.unescape().map_err(|e| KnowledgeError::Ingest(e.to_string()))?)
            }
            Ok(Event::End(tag)) if tag.name().as_ref() == b"w:p" => text.push('\n'),
            Ok(Event::Eof) => break,
            Err(e) => return Err(KnowledgeError::Ingest(format!("invalid DOCX XML: {e}"))),
            _ => {}
        }
    }
    Ok(text)
}

fn csv_text(bytes: &[u8]) -> Result<String, KnowledgeError> {
    let mut reader = csv::Reader::from_reader(bytes);
    let headers = reader.headers().map_err(|e| KnowledgeError::Ingest(e.to_string()))?.clone();
    let mut lines = Vec::new();
    for row in reader.records() {
        let row = row.map_err(|e| KnowledgeError::Ingest(e.to_string()))?;
        let line =
            headers.iter().zip(row.iter()).map(|(key, value)| format!("{key}: {value}")).collect::<Vec<_>>().join("; ");
        lines.push(line);
    }
    Ok(lines.join("\n"))
}

fn html(name: &str, bytes: &[u8]) -> Result<ParsedDocument, KnowledgeError> {
    let raw = std::str::from_utf8(bytes).map_err(|e| KnowledgeError::Ingest(e.to_string()))?;
    let page = Html::parse_document(raw);
    let title_selector = Selector::parse("title").map_err(|e| KnowledgeError::Ingest(e.to_string()))?;
    let body_selector = Selector::parse("body").map_err(|e| KnowledgeError::Ingest(e.to_string()))?;
    let title = page
        .select(&title_selector)
        .next()
        .map(|title| title.text().collect::<String>())
        .unwrap_or_else(|| name.into());
    let text =
        page.select(&body_selector).next().map(|body| body.text().collect::<Vec<_>>().join(" ")).unwrap_or_default();
    Ok(ParsedDocument { title, sections: vec![Section { heading: None, text }] })
}

fn markdown(name: &str, raw: &str) -> ParsedDocument {
    let mut title = name.to_owned();
    let mut heading = None;
    let mut content = String::new();
    let mut sections = Vec::new();
    for line in raw.lines() {
        if let Some(label) = line.strip_prefix('#').map(|s| s.trim_start_matches('#').trim()).filter(|s| !s.is_empty())
        {
            if !content.trim().is_empty() {
                sections.push(Section { heading: heading.take(), text: content.trim().into() });
                content.clear();
            }
            if title == name {
                title = label.into();
            }
            heading = Some(label.into());
        } else {
            content.push_str(line);
            content.push('\n');
        }
    }
    if !content.trim().is_empty() {
        sections.push(Section { heading, text: content.trim().into() });
    }
    ParsedDocument { title, sections }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn k01_markdown_preserves_headings() {
        let parsed = parse("guide.md", "text/markdown", b"# Guide\nIntro\n## Refunds\n30 days").unwrap();
        assert_eq!(parsed.title, "Guide");
        assert_eq!(parsed.sections[1].heading.as_deref(), Some("Refunds"));
    }

    #[test]
    fn k01_html_and_csv_extract_text() {
        let page = parse("page.html", "text/html", b"<title>Help</title><body><p>Policy</p></body>").unwrap();
        assert_eq!(page.title, "Help");
        assert!(page.sections[0].text.contains("Policy"));
        let csv = parse("rows.csv", "text/csv", b"name,policy\nAna,Refunds").unwrap();
        assert!(csv.sections[0].text.contains("policy: Refunds"));
    }

    #[test]
    fn k01_pdf_and_docx_fixtures_extract_text() {
        let pdf = parse("policy.pdf", "application/pdf", include_bytes!("../../tests/fixtures/policy.pdf")).unwrap();
        assert!(pdf.sections[0].text.contains("Refund policy"));
        let docx = parse(
            "policy.docx",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            include_bytes!("../../tests/fixtures/policy.docx"),
        )
        .unwrap();
        assert!(docx.sections[0].text.contains("Refund policy"));
    }
}
