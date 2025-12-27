#![no_main]

use arbitrary::Arbitrary;
use epub_builder::{EpubBuilder, EpubContent, EpubVersion, ZipLibrary};
use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

/// Fuzz input representing all metadata fields that can be set on an EPUB
#[derive(Debug, Arbitrary)]
struct FuzzMetadata {
    title: String,
    authors: Vec<FuzzAuthor>,
    publisher: Option<String>,
    descriptions: Vec<String>,
    subjects: Vec<String>,
    license: Option<String>,
    generator: String,
    toc_name: String,
    lang: String,
    series_name: Option<String>,
    series_number: u32,
    accessibility_summary: Option<String>,
    // Custom OPF metadata - this is a key vulnerability area
    custom_meta: Vec<FuzzMeta>,
    // Content titles (for guide/spine)
    content_title: String,
    // Use EPUB v2 or v3
    use_v3: bool,
}

#[derive(Debug, Arbitrary)]
struct FuzzAuthor {
    name: String,
    name_last_first: String,
}

#[derive(Debug, Arbitrary)]
struct FuzzMeta {
    name: String,
    content: String,
}

/// Validate XML by attempting to parse it
fn validate_xml(xml: &[u8]) -> Result<(), String> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(format!(
                    "XML parse error at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ));
            }
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

/// Extract content.opf from the generated EPUB zip
fn extract_opf(epub_bytes: &[u8]) -> Option<Vec<u8>> {
    use std::io::Read;

    let cursor = Cursor::new(epub_bytes);
    let mut archive = zip::ZipArchive::new(cursor).ok()?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).ok()?;
        if file.name().ends_with("content.opf") {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).ok()?;
            return Some(contents);
        }
    }
    None
}

/// Extract toc.ncx from the generated EPUB zip
fn extract_toc_ncx(epub_bytes: &[u8]) -> Option<Vec<u8>> {
    use std::io::Read;

    let cursor = Cursor::new(epub_bytes);
    let mut archive = zip::ZipArchive::new(cursor).ok()?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).ok()?;
        if file.name().ends_with("toc.ncx") {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).ok()?;
            return Some(contents);
        }
    }
    None
}

/// Extract nav.xhtml from the generated EPUB zip
fn extract_nav(epub_bytes: &[u8]) -> Option<Vec<u8>> {
    use std::io::Read;

    let cursor = Cursor::new(epub_bytes);
    let mut archive = zip::ZipArchive::new(cursor).ok()?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).ok()?;
        if file.name().ends_with("nav.xhtml") {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).ok()?;
            return Some(contents);
        }
    }
    None
}

fuzz_target!(|input: FuzzMetadata| {
    // Limit string lengths to avoid OOM
    if input.title.len() > 1000
        || input.authors.len() > 10
        || input.descriptions.len() > 10
        || input.subjects.len() > 10
        || input.custom_meta.len() > 10
    {
        return;
    }

    for author in &input.authors {
        if author.name.len() > 500 || author.name_last_first.len() > 500 {
            return;
        }
    }

    for meta in &input.custom_meta {
        if meta.name.len() > 500 || meta.content.len() > 500 {
            return;
        }
    }

    // Build EPUB
    let mut output = Vec::new();
    let result = (|| -> Result<(), epub_builder::Error> {
        let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

        // Set version
        if input.use_v3 {
            builder.epub_version(EpubVersion::V30);
        } else {
            builder.epub_version(EpubVersion::V20);
        }

        // Set all metadata fields
        builder.set_title(&input.title);
        builder.set_generator(&input.generator);
        builder.set_toc_name(&input.toc_name);
        builder.set_lang(&input.lang);

        if let Some(ref publisher) = input.publisher {
            builder.set_publisher(publisher);
        }

        if let Some(ref license) = input.license {
            builder.set_license(license);
        }

        // Authors
        for author in &input.authors {
            let _ = builder.add_author(epub_builder::Author {
                name: author.name.clone(),
                name_last_first: author.name_last_first.clone(),
            });
        }

        // Descriptions
        for desc in &input.descriptions {
            builder.add_description(desc);
        }

        // Subjects
        for subject in &input.subjects {
            builder.add_subject(subject);
        }

        // Series
        if let Some(ref series_name) = input.series_name {
            builder.set_series(epub_builder::Series::new(series_name, input.series_number));
        }

        // Accessibility summary
        if let Some(ref summary) = input.accessibility_summary {
            let mut access = epub_builder::AccessibilityMetadata::new();
            access.accessibility_summary = Some(summary.clone());
            builder.set_accessibility_metadata(access);
        }

        // Custom OPF metadata - HIGH RISK for XML injection
        for meta in &input.custom_meta {
            builder.add_metadata_opf(epub_builder::MetadataOpf {
                name: meta.name.clone(),
                content: meta.content.clone(),
            });
        }

        // Add minimal content so EPUB is valid
        let content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>Test</title></head>
<body><p>Test content</p></body>
</html>"#
        );

        builder.add_content(
            EpubContent::new("chapter1.xhtml", content.as_bytes())
                .title(&input.content_title)
                .reftype(epub_builder::ReferenceType::Text),
        )?;

        builder.generate(&mut output)?;
        Ok(())
    })();

    // If EPUB generation succeeded, validate the XML files
    if result.is_ok() && !output.is_empty() {
        // Validate content.opf
        if let Some(opf) = extract_opf(&output) {
            if let Err(e) = validate_xml(&opf) {
                // Found an XML parsing error!
                panic!(
                    "Invalid XML in content.opf: {}\n\nGenerated OPF:\n{}",
                    e,
                    String::from_utf8_lossy(&opf)
                );
            }
        }

        // Validate toc.ncx
        if let Some(ncx) = extract_toc_ncx(&output) {
            if let Err(e) = validate_xml(&ncx) {
                panic!(
                    "Invalid XML in toc.ncx: {}\n\nGenerated NCX:\n{}",
                    e,
                    String::from_utf8_lossy(&ncx)
                );
            }
        }

        // Validate nav.xhtml (EPUB 3 only)
        if let Some(nav) = extract_nav(&output) {
            if let Err(e) = validate_xml(&nav) {
                panic!(
                    "Invalid XML in nav.xhtml: {}\n\nGenerated NAV:\n{}",
                    e,
                    String::from_utf8_lossy(&nav)
                );
            }
        }
    }
});
