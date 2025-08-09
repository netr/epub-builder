use epub_builder::{
    AccessMode, AccessModeSufficient, AccessibilityFeature, AccessibilityHazard,
    AccessibilityMetadata, Author, EpubBuilder, EpubContent, ReferenceType, Result, ZipLibrary,
};

use std::env;
use std::fs::File;

/// Example demonstrating how to use accessibility features in EPUB generation
fn run() -> Result<()> {
    env_logger::init();

    // Create output file
    let curr_dir = env::current_dir().unwrap();
    let out_file = curr_dir.join("accessible_epub_example.epub");
    log::debug!("Writing accessible EPUB to: {}", &out_file.display());
    let writer = File::create(out_file).unwrap();

    // Sample content for our accessible EPUB
    let chapter_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head>
    <title>Chapter 1: Introduction</title>
</head>
<body>
    <h1>Chapter 1: Introduction to Accessibility</h1>
    <p>This is an example of an accessible EPUB document.</p>
    <img src="diagram.png" alt="A diagram showing the relationship between different accessibility standards" />
    <p>The content includes proper headings, alternative text for images, and follows WCAG guidelines.</p>
    <h2>Section 1.1: Basic Principles</h2>
    <p>Accessibility ensures that content can be perceived, understood, navigated, and interacted with by people with disabilities.</p>
</body>
</html>"#;

    let dummy_image = "Not really a PNG image - placeholder for accessibility diagram";
    let dummy_css = r#"
body { 
    font-family: Arial, sans-serif; 
    line-height: 1.6; 
    margin: 2em;
}
h1, h2, h3 { 
    color: #333; 
    margin-top: 1.5em;
}
img {
    max-width: 100%;
    height: auto;
}
"#;

    // Create a new EpubBuilder
    let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

    // Set basic metadata
    builder
        .add_author(Author::new("Jane Accessibility", "Accessibility, Jane"))?
        .publisher("Accessible Publishing Co.")?
        .metadata("title", "Guide to Accessible EPUBs")?
        .metadata("generator", "epub-builder accessibility example")?
        .epub_version(epub_builder::EpubVersion::V30)
        .stylesheet(dummy_css.as_bytes())?;

    // Create comprehensive accessibility metadata
    let mut accessibility = AccessibilityMetadata::new();

    // Set access modes - this content can be perceived visually and textually
    accessibility.access_modes = vec![AccessMode::Visual, AccessMode::Textual];

    // No accessibility hazards
    accessibility.accessibility_hazards = vec![AccessibilityHazard::None];

    // Content can be consumed through text alone or combined visual/textual
    accessibility.access_mode_sufficient = vec![
        AccessModeSufficient::Textual,
        AccessModeSufficient::TextualVisual,
    ];

    // List the accessibility features available
    accessibility.accessibility_features = vec![
        AccessibilityFeature::StructuralNavigation,
        AccessibilityFeature::TableOfContents,
        AccessibilityFeature::ReadingOrder,
        AccessibilityFeature::AlternativeText,
    ];

    // Set a custom accessibility summary
    accessibility.accessibility_summary = Some(
        "This publication has been created to demonstrate accessibility features in EPUB. \
         It includes proper heading structure, alternative text for images, and follows \
         WCAG 2.1 AA guidelines. All images essential to understanding the content \
         include descriptive alternative text."
            .to_string(),
    );

    // Apply the accessibility metadata to the builder
    builder.set_accessibility_metadata(accessibility);

    // Add cover image with alt text consideration
    // builder.add_cover_image("cover.png", dummy_image.as_bytes(), "image/png")?;
    builder.add_resource_with_id(
        "cover.png",
        dummy_image.as_bytes(),
        "image/png",
        "cover-image",
    )?;

    // Add the main content
    builder
        .add_content(
            EpubContent::new("chapter1.xhtml", chapter_content.as_bytes())
                .title("Chapter 1: Introduction to Accessibility")
                .properties("boobies")
                .reftype(ReferenceType::Text),
        )?
        // Add the image referenced in the content
        .add_resource("diagram.png", dummy_image.as_bytes(), "image/png")?;

    // Set up table of contents
    builder.inline_toc();

    // Generate the EPUB file
    builder.generate(writer)?;
    Ok(())
}

fn main() {
    match run() {
        Ok(_) => println!("Example completed successfully!"),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
