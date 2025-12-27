#![no_main]

use arbitrary::Arbitrary;
use epub_builder::{EpubBuilder, EpubContent, EpubVersion, TocElement, ZipLibrary};
use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

/// Fuzz input for TOC elements
#[derive(Debug, Arbitrary)]
struct FuzzToc {
    entries: Vec<FuzzTocEntry>,
    use_v3: bool,
}

#[derive(Debug, Arbitrary)]
struct FuzzTocEntry {
    title: String,
    url: String,
    children: Vec<FuzzTocChild>,
}

#[derive(Debug, Arbitrary)]
struct FuzzTocChild {
    title: String,
    url: String,
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

/// Extract a file from the EPUB zip by suffix
fn extract_file(epub_bytes: &[u8], suffix: &str) -> Option<Vec<u8>> {
    use std::io::Read;

    let cursor = Cursor::new(epub_bytes);
    let mut archive = zip::ZipArchive::new(cursor).ok()?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).ok()?;
        if file.name().ends_with(suffix) {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).ok()?;
            return Some(contents);
        }
    }
    None
}

fuzz_target!(|input: FuzzToc| {
    // Limit to avoid OOM
    if input.entries.len() > 20 {
        return;
    }

    for entry in &input.entries {
        if entry.title.len() > 500 || entry.url.len() > 500 || entry.children.len() > 10 {
            return;
        }
        for child in &entry.children {
            if child.title.len() > 500 || child.url.len() > 500 {
                return;
            }
        }
    }

    let mut output = Vec::new();
    let result = (|| -> Result<(), epub_builder::Error> {
        let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

        if input.use_v3 {
            builder.epub_version(EpubVersion::V30);
        } else {
            builder.epub_version(EpubVersion::V20);
        }

        builder.set_title("Fuzz Test");

        // Add chapters with fuzzed TOC entries
        for (i, entry) in input.entries.iter().enumerate() {
            let content = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>{}</title></head>
<body><p>Content {}</p></body>
</html>"#,
                i, i
            );

            let filename = format!("chapter{}.xhtml", i);

            // Create TOC element with potential nested children
            let mut toc_elem = TocElement::new(&filename, &entry.title);

            for (j, child) in entry.children.iter().enumerate() {
                let child_elem = TocElement::new(
                    format!("{}#section{}", filename, j),
                    &child.title,
                );
                toc_elem = toc_elem.child(child_elem);
            }

            builder.add_content(
                EpubContent::new(&filename, content.as_bytes())
                    .title(&entry.title)
                    .reftype(epub_builder::ReferenceType::Text),
            )?;

            builder.inline_toc();
        }

        builder.generate(&mut output)?;
        Ok(())
    })();

    if result.is_ok() && !output.is_empty() {
        // Validate content.opf
        if let Some(opf) = extract_file(&output, "content.opf") {
            if let Err(e) = validate_xml(&opf) {
                panic!(
                    "Invalid XML in content.opf: {}\n\nInput: {:?}\n\nGenerated OPF:\n{}",
                    e,
                    input,
                    String::from_utf8_lossy(&opf)
                );
            }
        }

        // Validate toc.ncx
        if let Some(ncx) = extract_file(&output, "toc.ncx") {
            if let Err(e) = validate_xml(&ncx) {
                panic!(
                    "Invalid XML in toc.ncx: {}\n\nInput: {:?}\n\nGenerated NCX:\n{}",
                    e,
                    input,
                    String::from_utf8_lossy(&ncx)
                );
            }
        }

        // Validate nav.xhtml
        if let Some(nav) = extract_file(&output, "nav.xhtml") {
            if let Err(e) = validate_xml(&nav) {
                panic!(
                    "Invalid XML in nav.xhtml: {}\n\nInput: {:?}\n\nGenerated NAV:\n{}",
                    e,
                    input,
                    String::from_utf8_lossy(&nav)
                );
            }
        }
    }
});
