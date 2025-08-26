// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::templates;
use crate::toc::{Toc, TocElement};
use crate::zip::Zip;
use crate::ReferenceType;
use crate::Result;
use crate::{common, EpubContent};

use std::io;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;
use upon::Engine;

/// Represents the EPUB version.
///
/// Currently, this library supports EPUB 2.0.1 and 3.0.1.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Eq)]
pub enum EpubVersion {
    /// EPUB 2.0.1 format
    V20,
    /// EPUB 3.0.1 format
    V30,
}

/// The page-progression-direction attribute of spine is a global attribute and
/// therefore defines the pagination flow of the book as a whole.
#[derive(Debug, Copy, Clone, Default)]
pub enum PageDirection {
    /// Left to right
    #[default]
    Ltr,
    /// Right to left
    Rtl,
}

/// Represents the EPUB `<meta>` content inside `content.opf` file.
///
/// <meta name="" content="">
///
#[derive(Debug)]
pub struct MetadataOpf {
    /// Name of the `<meta>` tag
    pub name: String,
    /// Content of the `<meta>` tag
    pub content: String,
}

impl MetadataOpf {
    /// Create new instance
    ///
    ///
    pub fn new(&self, meta_name: String, meta_content: String) -> Self {
        Self {
            name: meta_name,
            content: meta_content,
        }
    }
}

impl ToString for PageDirection {
    fn to_string(&self) -> String {
        match &self {
            PageDirection::Rtl => "rtl".into(),
            PageDirection::Ltr => "ltr".into(),
        }
    }
}

impl FromStr for PageDirection {
    type Err = crate::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_ref() {
            "rtl" => Ok(PageDirection::Rtl),
            "ltr" => Ok(PageDirection::Ltr),
            _ => Err(crate::Error::PageDirectionError(s)),
        }
    }
}

/// Represents an author of the EPUB book.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Author {
    /// Name of the author
    pub name: String,
    /// Name of the author in last, first format
    pub name_last_first: String,
}

impl Author {
    /// Create a new author
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the author, e.g. "John Doe"
    pub fn new<S: Into<String>>(name: S, name_last_first: S) -> Self {
        let name = name.into();
        let name_last_first = name_last_first.into();
        Author {
            name,
            name_last_first,
        }
    }
}
/// Accessibility access modes describing the primary sensory modalities through which
/// the content can be perceived (as defined in schema.org / EPUB a11y metadata).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessMode {
    /// Content can be perceived visually
    Visual,
    /// Content can be perceived textually
    Textual,
    /// Content can be perceived auditorily
    Auditory,
    /// Content can be perceived tactilely
    Tactile,
}

impl ToString for AccessMode {
    fn to_string(&self) -> String {
        match self {
            AccessMode::Visual => "visual".to_string(),
            AccessMode::Textual => "textual".to_string(),
            AccessMode::Auditory => "auditory".to_string(),
            AccessMode::Tactile => "tactile".to_string(),
        }
    }
}

/// Accessibility hazards for EPUB content
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessibilityHazard {
    /// No accessibility hazards
    None,
    /// Contains flashing content
    Flashing,
    /// Contains motion simulation
    MotionSimulation,
    /// Contains sound content
    Sound,
    /// Unknown hazards
    Unknown,
}

impl ToString for AccessibilityHazard {
    fn to_string(&self) -> String {
        match self {
            AccessibilityHazard::None => "none".to_string(),
            AccessibilityHazard::Flashing => "flashing".to_string(),
            AccessibilityHazard::MotionSimulation => "motionSimulation".to_string(),
            AccessibilityHazard::Sound => "sound".to_string(),
            AccessibilityHazard::Unknown => "unknown".to_string(),
        }
    }
}

/// Sufficient access modes for EPUB content
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessModeSufficient {
    /// Textual access is sufficient
    Textual,
    /// Visual access is sufficient
    Visual,
    /// Combined textual and visual access
    TextualVisual,
    /// Auditory access is sufficient
    Auditory,
    /// Tactile access is sufficient
    Tactile,
}

impl ToString for AccessModeSufficient {
    fn to_string(&self) -> String {
        match self {
            AccessModeSufficient::Textual => "textual".to_string(),
            AccessModeSufficient::Visual => "visual".to_string(),
            AccessModeSufficient::TextualVisual => "textual,visual".to_string(),
            AccessModeSufficient::Auditory => "auditory".to_string(),
            AccessModeSufficient::Tactile => "tactile".to_string(),
        }
    }
}

/// Accessibility features available in EPUB content
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessibilityFeature {
    /// Structural navigation elements
    StructuralNavigation,
    /// Table of contents
    TableOfContents,
    /// Reading order
    ReadingOrder,
    /// Alternative text for images
    AlternativeText,
    /// Audio descriptions
    AudioDescription,
    /// Captions
    Captions,
    /// Sign language interpretation
    SignLanguage,
    /// Synchronized audio text
    SynchronizedAudioText,
    /// Annotations
    Annotations,
    /// Bookmarks
    Bookmarks,
    /// Index
    Index,
    /// Print equivalent page numbers
    PrintPageNumbers,
}

impl ToString for AccessibilityFeature {
    fn to_string(&self) -> String {
        match self {
            AccessibilityFeature::StructuralNavigation => "structuralNavigation".to_string(),
            AccessibilityFeature::TableOfContents => "tableOfContents".to_string(),
            AccessibilityFeature::ReadingOrder => "readingOrder".to_string(),
            AccessibilityFeature::AlternativeText => "alternativeText".to_string(),
            AccessibilityFeature::AudioDescription => "audioDescription".to_string(),
            AccessibilityFeature::Captions => "captions".to_string(),
            AccessibilityFeature::SignLanguage => "signLanguage".to_string(),
            AccessibilityFeature::SynchronizedAudioText => "synchronizedAudioText".to_string(),
            AccessibilityFeature::Annotations => "annotations".to_string(),
            AccessibilityFeature::Bookmarks => "bookmarks".to_string(),
            AccessibilityFeature::Index => "index".to_string(),
            AccessibilityFeature::PrintPageNumbers => "printPageNumbers".to_string(),
        }
    }
}

/// Accessibility metadata for EPUB content
///
/// This struct contains optional accessibility information that can be included
/// in the EPUB metadata to improve accessibility compliance.
#[derive(Debug, Default)]
pub struct AccessibilityMetadata {
    /// Access modes available for the content
    pub access_modes: Vec<AccessMode>,
    /// Accessibility hazards present in the content
    pub accessibility_hazards: Vec<AccessibilityHazard>,
    /// Sufficient access modes for consuming the content
    pub access_mode_sufficient: Vec<AccessModeSufficient>,
    /// Accessibility features available in the content
    pub accessibility_features: Vec<AccessibilityFeature>,
    /// Summary of accessibility features and compliance
    pub accessibility_summary: Option<String>,
}

/// Series information for EPUB books that are part of a series
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Series {
    /// Name of the series
    pub name: String,
    /// Position/number of this book in the series
    pub book_number: u32,
}

impl Series {
    /// Create a new series
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the series, e.g. "A Riley Thomas Mystery"
    /// * `book_number`: the position of this book in the series, e.g. 2
    pub fn new<S: Into<String>>(name: S, book_number: u32) -> Self {
        Self {
            name: name.into(),
            book_number,
        }
    }
}

impl AccessibilityMetadata {
    /// Creates a new AccessibilityMetadata with default values for a typical EPUB
    pub fn new() -> Self {
        Self {
            access_modes: vec![AccessMode::Textual],
            accessibility_hazards: vec![AccessibilityHazard::None],
            access_mode_sufficient: vec![
                AccessModeSufficient::Textual,
            ],
            accessibility_features: vec![
                AccessibilityFeature::StructuralNavigation,
                AccessibilityFeature::ReadingOrder,
            ],
            accessibility_summary: Some(
                "The structure and markup of this publication follow WCAG guidelines. Images used in this publication have not been verified to follow these guidelines. Images integral to the content include alternate text descriptions. This summary has been automatically generated based on the content of this publication.".to_string()
            ),
        }
    }

    /// Creates an empty AccessibilityMetadata
    pub fn empty() -> Self {
        Self::default()
    }
}

/// EPUB Metadata
#[derive(Debug)]
pub struct Metadata {
    pub title: String,
    pub author: Vec<Author>,
    pub publisher: Option<String>,
    pub isbn: Option<String>,
    pub lang: String,
    pub direction: PageDirection,
    pub generator: String,
    pub toc_name: String,
    pub description: Vec<String>,
    pub subject: Vec<String>,
    pub license: Option<String>,
    pub date_published: Option<chrono::DateTime<chrono::Utc>>,
    pub date_modified: Option<chrono::DateTime<chrono::Utc>>,
    pub uuid: Option<uuid::Uuid>,
    pub lacuna_version: Option<String>,
    pub lacuna_target_platform: Option<String>,
    /// Accessibility metadata for the EPUB
    pub accessibility_metadata: AccessibilityMetadata,
    /// Series information for the EPUB
    pub series: Option<Series>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            title: String::new(),
            author: vec![],
            publisher: None,
            isbn: None,
            lang: String::from("en"),
            direction: PageDirection::default(),
            generator: String::from("Rust EPUB library"),
            toc_name: String::from("Table Of Contents"),
            description: vec![],
            subject: vec![],
            license: None,
            date_published: None,
            date_modified: None,
            uuid: None,
            lacuna_version: None,
            lacuna_target_platform: Some("Generic".to_string()),
            accessibility_metadata: AccessibilityMetadata::new(),
            series: None,
        }
    }
}

/// A file added in the EPUB
#[derive(Debug)]
struct Content {
    pub file: String,
    pub mime: String,
    pub itemref: bool,
    pub cover: bool,
    pub reftype: Option<ReferenceType>,
    pub title: String,
    pub include_in_guide: bool,
    pub id: Option<String>,
    pub properties: Option<String>,
}

impl Content {
    /// Create a new content file
    pub fn new<S1, S2>(file: S1, mime: S2) -> Content
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Content {
            file: file.into(),
            mime: mime.into(),
            itemref: false,
            cover: false,
            reftype: None,
            title: String::new(),
            include_in_guide: true,
            id: None,
            properties: None,
        }
    }
}

/// Epub Builder
///
/// The main struct you'll need to use in this library. It is first created using
/// a wrapper to zip files; then you add content to it, and finally you generate
/// the EPUB file by calling the `generate` method.
///
/// ```
/// use epub_builder::EpubBuilder;
/// use epub_builder::ZipCommand;
/// use std::io;
///
/// // "Empty" EPUB file
/// let mut builder = EpubBuilder::new(ZipCommand::new().unwrap()).unwrap();
/// builder.metadata("title", "Empty EPUB").unwrap();
/// builder.metadata("author", "Ann 'Onymous").unwrap();
/// builder.generate(&mut io::stdout()).unwrap();
/// ```
#[derive(Debug)]
pub struct EpubBuilder<Z: Zip> {
    version: EpubVersion,
    direction: PageDirection,
    zip: Z,
    files: Vec<Content>,
    metadata: Metadata,
    toc: Toc,
    stylesheet: bool,
    inline_toc: bool,
    escape_html: bool,
    meta_opf: Vec<MetadataOpf>,
}

impl<Z: Zip> EpubBuilder<Z> {
    /// Create a new default EPUB Builder
    pub fn new(zip: Z) -> Result<EpubBuilder<Z>> {
        let mut epub = EpubBuilder {
            version: EpubVersion::V20,
            direction: PageDirection::Ltr,
            zip,
            files: vec![],
            metadata: Metadata::default(),
            toc: Toc::new(),
            stylesheet: false,
            inline_toc: false,
            escape_html: true,
            meta_opf: Vec::new(),
        };

        epub.zip
            .write_file("META-INF/container.xml", templates::CONTAINER)?;
        epub.zip.write_file(
            "META-INF/com.apple.ibooks.display-options.xml",
            templates::IBOOKS,
        )?;

        Ok(epub)
    }

    /// Set EPUB version (default: V20)
    ///
    /// Supported versions are:
    ///
    /// * `V20`: EPUB 2.0.1
    /// * 'V30`: EPUB 3.0.1
    pub fn epub_version(&mut self, version: EpubVersion) -> &mut Self {
        self.version = version;
        self
    }

    /// Set EPUB Direction (default: Ltr)
    ///
    /// * `Ltr`: Left-To-Right
    /// * `Rtl`: Right-To-Left
    ///
    ///
    pub fn epub_direction(&mut self, direction: PageDirection) -> &mut Self {
        self.direction = direction;
        self
    }

    /// Add custom <meta> to `content.opf`
    /// Syntax: `self.add_metadata_opf(name, content)`
    ///
    /// ### Example
    /// If you wanna add `<meta name="primary-writing-mode" content="vertical-rl"/>` into `content.opf`
    ///
    /// ```rust
    /// use epub_builder::EpubBuilder;
    /// use epub_builder::ZipCommand;
    /// use epub_builder::MetadataOpf;
    /// let mut builder = EpubBuilder::new(ZipCommand::new().unwrap()).unwrap();

    /// builder.add_metadata_opf(MetadataOpf {
    ///     name: String::from("primary-writing-mode"),
    ///     content: String::from("vertical-rl")
    /// });
    /// ```
    ///
    pub fn add_metadata_opf(&mut self, item: MetadataOpf) -> &mut Self {
        self.meta_opf.push(item);
        self
    }

    /// Set some EPUB metadata
    ///
    /// For most metadata, this function will replace the existing metadata, but for subject, cteator and identifier who
    /// can have multiple values, it will add data to the existing data, unless the empty string "" is passed, in which case
    /// it will delete existing data for this key.
    ///
    /// # Valid keys used by the EPUB builder
    ///
    /// * `author`: author(s) of the book;
    /// * `title`: title of the book;
    /// * `lang`: the language of the book, quite important as EPUB renderers rely on it
    ///   for e.g. hyphenating words.
    /// * `generator`: generator of the book (should be your program name);
    /// * `toc_name`: the name to use for table of contents (by default, "Table of Contents");
    /// * `subject`;
    /// * `description`;
    /// * `license`.

    pub fn metadata<S1, S2>(&mut self, key: S1, value: S2) -> Result<&mut Self>
    where
        S1: AsRef<str>,
        S2: Into<String>,
    {
        match key.as_ref() {
            "title" => self.metadata.title = value.into(),
            "lang" => self.metadata.lang = value.into(),
            "direction" => self.metadata.direction = PageDirection::from_str(&value.into())?,
            "generator" => self.metadata.generator = value.into(),
            "isbn" => {
                let value = value.into();
                if value.is_empty() {
                    self.metadata.isbn = None;
                } else {
                    self.metadata.isbn = Some(value);
                }
            }
            "description" => {
                let value = value.into();
                if value.is_empty() {
                    self.metadata.description = vec![];
                } else {
                    self.metadata.description.push(value);
                }
            }
            "subject" => {
                let value = value.into();
                if value.is_empty() {
                    self.metadata.subject = vec![];
                } else {
                    self.metadata.subject.push(value);
                }
            }
            "lacuna_version" => {
                let value = value.into();
                if value.is_empty() {
                    self.metadata.lacuna_version = None;
                } else {
                    self.metadata.lacuna_version = Some(value);
                }
            }
            "lacuna_target_platform" => {
                let value = value.into();
                if value.is_empty() {
                    self.metadata.lacuna_target_platform = Some("Generic".to_string());
                } else {
                    self.metadata.lacuna_target_platform = Some(value);
                }
            }
            "license" => self.metadata.license = Some(value.into()),
            "toc_name" => self.metadata.toc_name = value.into(),
            s => Err(crate::Error::InvalidMetadataError(s.to_string()))?,
        }
        Ok(self)
    }

    /// Sets the Publisher of the EPUB
    pub fn set_publisher<S: Into<String>>(&mut self, value: S) {
        self.metadata.publisher = Some(value.into());
    }

    /// Sets the Publisher of the EPUB
    pub fn publisher<S: Into<String>>(&mut self, value: S) -> Result<&mut Self> {
        self.metadata.publisher = Some(value.into());
        Ok(self)
    }

    /// Sets the authors of the EPUB
    pub fn set_authors(&mut self, value: Vec<Author>) {
        self.metadata.author = value;
    }

    /// Add an author to the EPUB
    pub fn add_author<S: Into<Author>>(&mut self, value: S) -> Result<&mut Self> {
        self.metadata.author.push(value.into());
        Ok(self)
    }

    /// Remove all authors from EPUB
    pub fn clear_authors<S: Into<String>>(&mut self) {
        self.metadata.author.clear()
    }

    /// Set Accessibility metadata for the EPUB
    pub fn set_accessibility_metadata(&mut self, metadata: AccessibilityMetadata) {
        self.metadata.accessibility_metadata = metadata;
    }

    /// Sets Accessibility metadata for the EPUB
    pub fn accessibility_metadata(&mut self, metadata: AccessibilityMetadata) -> Result<&mut Self> {
        self.set_accessibility_metadata(metadata);
        Ok(self)
    }

    /// Set series metadata for the EPUB
    pub fn set_series(&mut self, series: Series) {
        self.metadata.series = Some(series);
    }

    /// Sets series metadata for the EPUB
    pub fn series(&mut self, series: Series) -> Result<&mut Self> {
        self.set_series(series);
        Ok(self)
    }

    /// Sets the title of the EPUB
    pub fn set_title<S: Into<String>>(&mut self, value: S) {
        self.metadata.title = value.into();
    }

    /// Tells whether fields should be HTML-escaped.
    ///
    /// * `true`: fields such as titles, description, and so on will be HTML-escaped everywhere (default)
    /// * `false`: fields will be left as is (letting you in charge of making
    /// sure they do not contain anything illegal, e.g. < and > characters)
    pub fn escape_html(&mut self, val: bool) {
        self.escape_html = val;
    }

    /// Sets the language of the EPUB
    ///
    /// This is quite important as EPUB renderers rely on it
    /// for e.g. hyphenating words.
    pub fn set_lang<S: Into<String>>(&mut self, value: S) {
        self.metadata.lang = value.into();
    }

    /// Sets the generator of the book (should be your program name)
    pub fn set_generator<S: Into<String>>(&mut self, value: S) {
        self.metadata.generator = value.into();
    }

    /// Sets the name to use for table of contents. This is by default, "Table of Contents"
    pub fn set_toc_name<S: Into<String>>(&mut self, value: S) {
        self.metadata.toc_name = value.into();
    }

    /// Sets and replaces the description of the EPUB
    pub fn set_description(&mut self, value: Vec<String>) {
        self.metadata.description = value;
    }

    /// Adds a line to the EPUB description
    pub fn add_description<S: Into<String>>(&mut self, value: S) {
        self.metadata.description.push(value.into());
    }

    /// Remove all description paragraphs from EPUB
    pub fn clear_description(&mut self) {
        self.metadata.description.clear();
    }

    /// Sets and replaces the subjects of the EPUB
    pub fn set_subjects(&mut self, value: Vec<String>) {
        self.metadata.subject = value;
    }

    /// Adds a value to the subjects
    pub fn add_subject<S: Into<String>>(&mut self, value: S) {
        self.metadata.subject.push(value.into());
    }

    /// Remove all the subjects from EPUB
    pub fn clear_subjects(&mut self) {
        self.metadata.subject.clear();
    }

    /// Sets the license under which this EPUB is distributed
    pub fn set_license<S: Into<String>>(&mut self, value: S) {
        self.metadata.license = Some(value.into());
    }

    /// Sets the publication date of the EPUB
    pub fn set_publication_date(&mut self, date_published: chrono::DateTime<chrono::Utc>) {
        self.metadata.date_published = Some(date_published);
    }
    /// Sets the date on which the EPUB was last modified.
    ///
    /// This value is part of the metadata. If this function is not called, the time at the
    /// moment of generation will be used instead.
    pub fn set_modified_date(&mut self, date_modified: chrono::DateTime<chrono::Utc>) {
        self.metadata.date_modified = Some(date_modified);
    }
    /// Sets the uuid used for the EPUB.
    ///
    /// This is useful for reproducibly generating epubs.
    pub fn set_uuid(&mut self, uuid: uuid::Uuid) {
        self.metadata.uuid = Some(uuid);
    }

    /// Sets stylesheet of the EPUB.
    ///
    /// This content will be written in a `stylesheet.css` file; it is used by
    /// some pages (such as nav.xhtml), you don't have use it in your documents though it
    /// makes sense to also do so.
    pub fn stylesheet<R: Read>(&mut self, content: R) -> Result<&mut Self> {
        self.add_resource("stylesheet.css", content, "text/css")?;
        self.stylesheet = true;
        Ok(self)
    }

    /// Adds an inline toc in the document.
    ///
    /// If this method is called it adds a page that contains the table of contents
    /// that appears in the document.
    ///
    /// The position where this table of contents will be inserted depends on when
    /// you call this method: if you call it before adding any content, it will be
    /// at the beginning, if you call it after, it will be at the end.
    pub fn inline_toc(&mut self) -> &mut Self {
        self.inline_toc = true;
        self.toc.add(TocElement::new(
            "toc.xhtml",
            self.metadata.toc_name.as_str(),
        ));
        let mut file = Content::new("toc.xhtml", "application/xhtml+xml");
        file.reftype = Some(ReferenceType::Toc);
        file.title = self.metadata.toc_name.clone();
        file.itemref = true;
        self.files.push(file);
        self
    }

    /// Add a resource to the EPUB file
    ///
    /// This resource can be a picture, a font, some CSS file, .... Unlike
    /// `add_content`, files added this way won't appear in the linear
    /// document.
    ///
    /// Note that these files will automatically be inserted into an `OEBPS` directory,
    /// so you don't need (and shouldn't) prefix your path with `OEBPS/`.
    ///
    /// # Arguments
    ///
    /// * `path`: the path where this file will be written in the EPUB OEBPS structure,
    ///   e.g. `data/image_0.png`
    /// * `content`: the resource to include
    /// * `mime_type`: the mime type of this file, e.g. "image/png".
    pub fn add_resource<R, P, S>(&mut self, path: P, content: R, mime_type: S) -> Result<&mut Self>
    where
        R: Read,
        P: AsRef<Path>,
        S: Into<String>,
    {
        self.zip
            .write_file(Path::new("OEBPS").join(path.as_ref()), content)?;
        log::debug!("Add resource: {:?}", path.as_ref().display());
        self.files.push(Content::new(
            format!("{}", path.as_ref().display()),
            mime_type,
        ));
        Ok(self)
    }

    /// Add a resource to the EPUB file with a custom ID.
    ///
    /// This is similar to `add_resource`, but allows you to specify a custom ID
    /// for the item in the manifest instead of having it auto-generated.
    ///
    /// # Examples
    ///
    /// ```
    /// # use epub_builder::{EpubBuilder, ZipLibrary};
    /// # let mut builder = EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();
    /// // Add a cover image with a specific ID
    /// builder.add_resource_with_id("images/cover.jpg",
    ///                               std::io::Cursor::new(vec![]),
    ///                               "image/jpeg",
    ///                               "cover-image").unwrap();
    /// // Then add the metadata to reference it
    /// builder.custom_opf_metadata("cover", "cover-image").unwrap();
    /// ```
    ///
    /// # Arguments
    ///
    /// * `path`: the path where this file will be written in the EPUB OEBPS structure
    /// * `content`: the resource to include
    /// * `mime_type`: the mime type of this file
    /// * `id`: the custom ID to use in the manifest
    pub fn add_resource_with_id<R, P, S, I>(
        &mut self,
        path: P,
        content: R,
        mime_type: S,
        id: I,
    ) -> Result<&mut Self>
    where
        R: Read,
        P: AsRef<Path>,
        S: Into<String>,
        I: Into<String>,
    {
        self.zip
            .write_file(Path::new("OEBPS").join(path.as_ref()), content)?;
        log::debug!("Add resource with ID: {:?}", path.as_ref().display());
        let mut file = Content::new(format!("{}", path.as_ref().display()), mime_type);
        file.id = Some(id.into());
        self.files.push(file);
        Ok(self)
    }

    /// Add a cover image to the EPUB.
    ///
    /// This works similarly to adding the image as a resource with the `add_resource`
    /// method, except, it signals it in the Manifest section so it is displayed as the
    /// cover by Ereaders
    pub fn add_cover_image<R, P, S>(
        &mut self,
        path: P,
        content: R,
        mime_type: S,
    ) -> Result<&mut Self>
    where
        R: Read,
        P: AsRef<Path>,
        S: Into<String>,
    {
        self.zip
            .write_file(Path::new("OEBPS").join(path.as_ref()), content)?;
        let mut file = Content::new(format!("{}", path.as_ref().display()), mime_type);
        file.cover = true;
        self.files.push(file);
        Ok(self)
    }

    /// Add a XHTML content file that will be added to the EPUB.
    ///
    /// # Examples
    ///
    /// ```
    /// # use epub_builder::{EpubBuilder, ZipLibrary, EpubContent};
    /// let content = "Some content";
    /// let mut builder = EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();
    /// // Add a chapter that won't be added to the Table of Contents
    /// builder.add_content(EpubContent::new("intro.xhtml", content.as_bytes())).unwrap();
    /// ```
    ///
    /// ```
    /// # use epub_builder::{EpubBuilder, ZipLibrary, EpubContent, TocElement};
    /// # let mut builder = EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();
    /// # let content = "Some content";
    /// // Sets the title of a chapter so it is added to the Table of contents
    /// // Also add information about its structure
    /// builder.add_content(EpubContent::new("chapter_1.xhtml", content.as_bytes())
    ///                      .title("Chapter 1")
    ///                      .child(TocElement::new("chapter_1.xhtml#1", "1.1"))).unwrap();
    /// ```
    ///
    /// ```
    /// # use epub_builder::{EpubBuilder, ZipLibrary, EpubContent};
    /// # let mut builder = EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();
    /// # let content = "Some content";
    /// // Add a section, by setting the level to 2 (instead of the default value 1)
    /// builder.add_content(EpubContent::new("section.xhtml", content.as_bytes())
    ///                      .title("Section 1")
    ///                      .level(2)).unwrap();
    /// ```
    ///
    /// Note that these files will automatically be inserted into an `OEBPS` directory,
    /// so you don't need (and shouldn't) prefix your path with `OEBPS/`.
    ///
    /// # See also
    ///
    /// * [`EpubContent`](struct.EpubContent.html)
    /// * the `add_resource` method, to add other resources in the EPUB file.
    pub fn add_content<R: Read>(&mut self, content: EpubContent<R>) -> Result<&mut Self> {
        self.zip.write_file(
            Path::new("OEBPS").join(content.toc.url.as_str()),
            content.content,
        )?;
        let mut file = Content::new(content.toc.url.as_str(), "application/xhtml+xml");
        file.itemref = true;
        file.reftype = content.reftype;
        file.include_in_guide = content.include_in_guide;
        file.properties = content.properties;
        if file.reftype.is_some() {
            file.title = content.toc.title.clone();
        }
        self.files.push(file);
        if !content.toc.title.is_empty() {
            self.toc.add(content.toc);
        }
        Ok(self)
    }

    /// Generate the EPUB file and write it to the writer
    ///
    /// # Example
    ///
    /// ```
    /// # use epub_builder::{EpubBuilder, ZipLibrary};
    /// let mut builder = EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();
    /// // Write the EPUB file into a Vec<u8>
    /// let mut epub: Vec<u8> = vec!();
    /// builder.generate(&mut epub).unwrap();
    /// ```
    pub fn generate<W: io::Write>(mut self, to: W) -> Result<()> {
        // If no styleesheet was provided, generate a dummy one
        // if !self.stylesheet {
        //     self.stylesheet(b"".as_ref())?;
        // }
        // Render content.opf
        let bytes = self.render_opf()?;
        self.zip.write_file("OEBPS/content.opf", &*bytes)?;
        // Render toc.ncx
        let bytes = self.render_toc()?;
        self.zip.write_file("OEBPS/toc.ncx", &*bytes)?;
        // Render nav.xhtml
        if self.version >= EpubVersion::V30 {
            let bytes = self.render_nav(true)?;
            self.zip.write_file("OEBPS/nav.xhtml", &*bytes)?;
        }
        // // Write inline toc if it needs to
        // if self.inline_toc {
        //     let bytes = self.render_nav(false)?;
        //     self.zip.write_file("OEBPS/toc.xhtml", &*bytes)?;
        // }

        self.zip.generate(to)?;
        Ok(())
    }

    /// Render content.opf file
    fn render_opf(&mut self) -> Result<Vec<u8>> {
        log::debug!("render_opf...");
        let mut optional: Vec<String> = Vec::new();
        for desc in &self.metadata.description {
            optional.push(format!(
                "<dc:description>{}</dc:description>",
                common::encode_html(desc, self.escape_html),
            ));
        }
        for subject in &self.metadata.subject {
            optional.push(format!(
                "<dc:subject>{}</dc:subject>",
                common::encode_html(subject, self.escape_html),
            ));
        }
        if let Some(ref rights) = self.metadata.license {
            optional.push(format!(
                "<dc:rights>{}</dc:rights>",
                common::encode_html(rights, self.escape_html),
            ));
        }
        for meta in &self.meta_opf {
            optional.push(format!(
                "<meta name=\"{}\" content=\"{}\"/>",
                common::encode_html(&meta.name, self.escape_html),
                common::encode_html(&meta.content, self.escape_html),
            ));
        }

        // Add accessibility metadata
        for access_mode in &self.metadata.accessibility_metadata.access_modes {
            optional.push(format!(
                "<meta name=\"schema:accessMode\" content=\"{}\"/>",
                common::encode_html(&access_mode.to_string(), self.escape_html),
            ));
        }

        for hazard in &self.metadata.accessibility_metadata.accessibility_hazards {
            optional.push(format!(
                "<meta name=\"schema:accessibilityHazard\" content=\"{}\"/>",
                common::encode_html(&hazard.to_string(), self.escape_html),
            ));
        }

        for sufficient in &self.metadata.accessibility_metadata.access_mode_sufficient {
            optional.push(format!(
                "<meta name=\"schema:accessModeSufficient\" content=\"{}\"/>",
                common::encode_html(&sufficient.to_string(), self.escape_html),
            ));
        }

        for feature in &self.metadata.accessibility_metadata.accessibility_features {
            optional.push(format!(
                "<meta name=\"schema:accessibilityFeature\" content=\"{}\"/>",
                common::encode_html(&feature.to_string(), self.escape_html),
            ));
        }

        if let Some(ref summary) = self.metadata.accessibility_metadata.accessibility_summary {
            optional.push(format!(
                "<meta name=\"schema:accessibilitySummary\" content=\"{}\"/>",
                common::encode_html(summary, self.escape_html),
            ));
        }

        // Add series metadata
        if let Some(ref series) = self.metadata.series {
            match self.version {
                EpubVersion::V30 => {
                    // EPUB 3.0 uses property-based metadata with refinements
                    optional.push(format!(
                        "<meta property=\"belongs-to-collection\" id=\"id-1\">{}</meta>",
                        common::encode_html(&series.name, self.escape_html),
                    ));
                    optional.push(
                        "<meta refines=\"#id-1\" property=\"collection-type\">series</meta>".to_string(),
                    );
                    optional.push(format!(
                        "<meta refines=\"#id-1\" property=\"group-position\">{}</meta>",
                        series.book_number,
                    ));
                }
                EpubVersion::V20 => {
                    // EPUB 2.0 uses name/content attributes for basic series support
                    optional.push(format!(
                        "<meta name=\"series\" content=\"{}\"/>",
                        common::encode_html(&series.name, self.escape_html),
                    ));
                    optional.push(format!(
                        "<meta name=\"series-position\" content=\"{}\"/>",
                        series.book_number,
                    ));
                }
            }
        }

        let date_modified = self
            .metadata
            .date_modified
            .unwrap_or_else(chrono::Utc::now)
            .format("%Y-%m-%dT%H:%M:%SZ");
        let date_published = self
            .metadata
            .date_published
            .map(|date| date.format("%Y-%m-%dT%H:%M:%SZ"));
        let uuid = uuid::fmt::Urn::from_uuid(self.metadata.uuid.unwrap_or_else(uuid::Uuid::new_v4))
            .to_string();

        let mut items: Vec<String> = Vec::new();
        let mut itemrefs: Vec<String> = Vec::new();
        let mut guide: Vec<String> = Vec::new();

        for content in &self.files {
            let id = if let Some(ref custom_id) = content.id {
                custom_id.clone()
            } else if content.cover {
                String::from("cover-image")
            } else {
                to_id(&content.file)
            };
            let mut properties_list: Vec<String> = Vec::new();
            if self.version >= EpubVersion::V30 && content.cover {
                properties_list.push("cover-image".to_string());
            }
            if self.version >= EpubVersion::V30 {
                if let Some(ref props) = content.properties {
                    for p in props.split_whitespace() {
                        if !properties_list.iter().any(|existing| existing == p) {
                            properties_list.push(p.to_string());
                        }
                    }
                }
            }
            let properties = if !properties_list.is_empty() {
                format!(
                    "properties=\"{}\" ",
                    html_escape::encode_double_quoted_attribute(&properties_list.join(" "))
                )
            } else {
                String::new()
            };
            if content.cover {
                optional.push(format!(
                    "<meta name=\"cover\" content=\"{}\"/>",
                    html_escape::encode_double_quoted_attribute(&id)
                ));
            }
            log::debug!("id={:?}, mime={:?}", id, content.mime);
            items.push(format!(
                "<item media-type=\"{mime}\" {properties}\
                        id=\"{id}\" href=\"{href}\"/>",
                properties = properties, // Not escaped: XML attributes above
                mime = html_escape::encode_double_quoted_attribute(&content.mime),
                id = html_escape::encode_double_quoted_attribute(&id),
                // in the zip the path is always with forward slashes, on windows it is with backslashes
                href =
                    html_escape::encode_double_quoted_attribute(&content.file.replace('\\', "/")),
            ));
            if content.itemref {
                itemrefs.push(format!(
                    "<itemref idref=\"{id}\"/>",
                    id = html_escape::encode_double_quoted_attribute(&id),
                ));
            }
            if let Some(reftype) = content.reftype {
                if content.include_in_guide {
                    use crate::ReferenceType::*;
                    let reftype = match reftype {
                        Cover => "cover",
                        TitlePage => "title-page",
                        Toc => "toc",
                        Index => "index",
                        Glossary => "glossary",
                        Acknowledgements => "acknowledgements",
                        Bibliography => "bibliography",
                        Colophon => "colophon",
                        Copyright => "copyright",
                        Dedication => "dedication",
                        Epigraph => "epigraph",
                        Foreword => "foreword",
                        Loi => "loi",
                        Lot => "lot",
                        Notes => "notes",
                        Preface => "preface",
                        Text => "text",
                    };
                    log::debug!("content = {:?}", &content);
                    guide.push(format!(
                        "<reference type=\"{reftype}\" title=\"{title}\" href=\"{href}\"/>",
                        reftype = html_escape::encode_double_quoted_attribute(&reftype),
                        title = html_escape::encode_double_quoted_attribute(&content.title),
                        href = html_escape::encode_double_quoted_attribute(&content.file),
                    ));
                }
            }
        }

        let data = {
            let mut authors: Vec<_> = vec![];
            for (i, author) in self.metadata.author.iter().enumerate() {
                let author = upon::value! {
                    id_attr: html_escape::encode_double_quoted_attribute(&i.to_string()),
                    name: common::encode_html(author.name.as_str(), self.escape_html),
                    name_last_first: common::encode_html(author.name_last_first.as_str(), self.escape_html)
                };
                authors.push(author);
            }
            upon::value! {
                author: authors,
                lang: html_escape::encode_text(&self.metadata.lang),
                isbn: common::encode_html(
                    self.metadata.isbn.as_deref().unwrap_or(""),
                    self.escape_html,
                ),
                publisher: common::encode_html(
                    self.metadata.publisher.as_deref().unwrap_or(""),
                    self.escape_html,
                ),
                direction: self.metadata.direction.to_string(),
                title: common::encode_html(&self.metadata.title, self.escape_html),
                generator_attr: html_escape::encode_double_quoted_attribute(&self.metadata.generator),
                toc_name: common::encode_html(&self.metadata.toc_name, self.escape_html),
                toc_name_attr: html_escape::encode_double_quoted_attribute(&self.metadata.toc_name),
                optional: common::indent(optional.join("\n"), 2),
                items: common::indent(items.join("\n"), 2), // Not escaped: XML content
                itemrefs: common::indent(itemrefs.join("\n"), 2), // Not escaped: XML content
                date_modified: html_escape::encode_text(&date_modified.to_string()),
                uuid: html_escape::encode_text(&uuid),
                lacuna_version: if let Some(ref version) = self.metadata.lacuna_version {
                    common::encode_html(version, self.escape_html)
                } else {
                    std::borrow::Cow::Borrowed("")
                },
                lacuna_target_platform: if let Some(ref platform) = self.metadata.lacuna_target_platform {
                    common::encode_html(platform, self.escape_html)
                } else {
                    std::borrow::Cow::Borrowed("")
                },
                guide: common::indent(guide.join("\n"), 2), // Not escaped: XML content
                date_published: if let Some(date) = date_published { date.to_string() } else { String::new() },
            }
        };

        let mut res: Vec<u8> = vec![];
        match self.version {
            EpubVersion::V20 => templates::v2::CONTENT_OPF
                .render(&Engine::new(), &data)
                .to_writer(&mut res),
            EpubVersion::V30 => templates::v3::CONTENT_OPF
                .render(&Engine::new(), &data)
                .to_writer(&mut res),
        }
        .map_err(|e| crate::Error::TemplateError {
            msg: "could not render template for content.opf".to_string(),
            cause: e.into(),
        })?;
        //.wrap_err("could not render template for content.opf")?;

        Ok(res)
    }

    /// Render toc.ncx
    fn render_toc(&mut self) -> Result<Vec<u8>> {
        let mut nav_points = String::new();

        nav_points.push_str(&self.toc.render_epub(self.escape_html));

        let data = upon::value! {
            toc_name: common::encode_html(&self.metadata.toc_name, self.escape_html),
            nav_points: nav_points
        };
        let mut res: Vec<u8> = vec![];
        templates::TOC_NCX
            .render(&Engine::new(), &data)
            .to_writer(&mut res)
            .map_err(|e| crate::Error::TemplateError {
                msg: "error rendering toc.ncx template".to_string(),
                cause: e.into(),
            })?;
        Ok(res)
    }

    /// Render nav.xhtml
    fn render_nav(&mut self, numbered: bool) -> Result<Vec<u8>> {
        let content = self.toc.render(numbered, self.escape_html);
        let mut landmarks: Vec<String> = Vec::new();
        if self.version > EpubVersion::V20 {
            for file in &self.files {
                // Only include items explicitly marked for inclusion in the guide / landmarks.
                // This preserves backwards compatibility because existing content defaults
                // include_in_guide to true (see Content::new). Any newly created content that
                // opts out by setting include_in_guide = false will now be skipped here.
                if !file.include_in_guide {
                    continue;
                }
                if let Some(ref reftype) = file.reftype {
                    use ReferenceType::*;
                    let reftype = match *reftype {
                        Cover => "cover",
                        Text => "bodymatter",
                        Toc => "toc",
                        Bibliography => "bibliography",
                        Epigraph => "epigraph",
                        Foreword => "foreword",
                        Preface => "preface",
                        Notes => "endnotes",
                        Loi => "loi",
                        Lot => "lot",
                        Colophon => "colophon",
                        TitlePage => "titlepage",
                        Index => "index",
                        Glossary => "glossary",
                        Copyright => "copyright-page",
                        Acknowledgements => "acknowledgements",
                        Dedication => "dedication",
                    };
                    if !file.title.is_empty() {
                        landmarks.push(format!(
                            "<li><a epub:type=\"{reftype}\" href=\"{href}\">\
                                {title}</a></li>",
                            reftype = html_escape::encode_double_quoted_attribute(&reftype),
                            href = html_escape::encode_double_quoted_attribute(&file.file),
                            title = common::encode_html(&file.title, self.escape_html),
                        ));
                    }
                }
            }
        }

        let data = upon::value! {
            content: content, // Not escaped: XML content
            toc_name: common::encode_html(&self.metadata.toc_name, self.escape_html),
            generator_attr: html_escape::encode_double_quoted_attribute(&self.metadata.generator),
            lang_attr: html_escape::encode_double_quoted_attribute(&self.metadata.lang),
            landmarks: if !landmarks.is_empty() {
                common::indent(
                    format!(
                        "<ol>\n{}\n</ol>",
                        common::indent(landmarks.join("\n"), 1), // Not escaped: XML content
                    ),
                    2,
                )
            } else {
                String::new()
            },
        };

        let mut res: Vec<u8> = vec![];
        match self.version {
            EpubVersion::V20 => templates::v2::NAV_XHTML
                .render(&Engine::new(), &data)
                .to_writer(&mut res),
            EpubVersion::V30 => templates::v3::NAV_XHTML
                .render(&Engine::new(), &data)
                .to_writer(&mut res),
        }
        .map_err(|e| crate::Error::TemplateError {
            msg: "error rendering nav.xhtml template".to_string(),
            cause: e.into(),
        })?;
        Ok(res)
    }
}

// The actual rules for ID are here - https://www.w3.org/TR/xml-names11/#NT-NCNameChar
// Ordering to to look as similar as possible to the W3 Recommendation ruleset
// Slightly more permissive, there are some that are invalid start chars, but this is ok.
fn is_id_char(c: char) -> bool {
    c.is_ascii_uppercase()
        || c == '_'
        || c.is_ascii_lowercase()
        || ('\u{C0}'..='\u{D6}').contains(&c)
        || ('\u{D8}'..='\u{F6}').contains(&c)
        || ('\u{F8}'..='\u{2FF}').contains(&c)
        || ('\u{370}'..='\u{37D}').contains(&c)
        || ('\u{37F}'..='\u{1FFF}').contains(&c)
        || ('\u{200C}'..='\u{200D}').contains(&c)
        || ('\u{2070}'..='\u{218F}').contains(&c)
        || ('\u{2C00}'..='\u{2FEF}').contains(&c)
        || ('\u{3001}'..='\u{D7FF}').contains(&c)
        || ('\u{F900}'..='\u{FDCF}').contains(&c)
        || ('\u{FDF0}'..='\u{FFFD}').contains(&c)
        || ('\u{10000}'..='\u{EFFFF}').contains(&c)
        || c == '-'
        || c == '.'
        || c.is_ascii_digit()
        || c == '\u{B7}'
        || ('\u{0300}'..='\u{036F}').contains(&c)
        || ('\u{203F}'..='\u{2040}').contains(&c)
}

// generate an id compatible string, replacing all none ID chars to underscores
fn to_id(s: &str) -> String {
    "id_".to_string() + &s.replace(|c: char| !is_id_char(c), "_")
}
