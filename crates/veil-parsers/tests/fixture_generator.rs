//! Generate PDF test fixtures.
//!
//! This module generates test PDF files for the parser tests.

use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream};
use std::fs::File;
use std::path::Path;

/// Generate a simple PDF with plain text.
pub fn generate_simple_pdf(path: &Path, text: &str) -> std::io::Result<()> {
    let mut doc = Document::with_version("1.5");

    // Create font
    let font_id = doc.new_object_id();
    doc.objects.insert(
        font_id,
        Object::Dictionary(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
        }),
    );

    // Create resources
    let resources_id = doc.new_object_id();
    doc.objects.insert(
        resources_id,
        Object::Dictionary(dictionary! {
            "Font" => dictionary! {
                "F1" => Object::Reference(font_id),
            },
        }),
    );

    // Create content
    let content_ops = create_text_operations(text, 72.0, 720.0);
    let content = Content {
        operations: content_ops,
    };
    let content_bytes = content.encode().unwrap_or_default();
    let content_stream = Stream::new(dictionary! {}, content_bytes);
    let content_id = doc.add_object(content_stream);

    // Create pages
    let pages_id = doc.new_object_id();

    // Create page
    let page_id = doc.new_object_id();
    doc.objects.insert(
        page_id,
        Object::Dictionary(dictionary! {
            "Type" => "Page",
            "Parent" => Object::Reference(pages_id),
            "MediaBox" => vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Integer(612),
                Object::Integer(792),
            ],
            "Contents" => Object::Reference(content_id),
            "Resources" => Object::Reference(resources_id),
        }),
    );

    // Create pages node
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => vec![Object::Reference(page_id)],
            "Count" => Object::Integer(1),
        }),
    );

    // Create catalog
    let catalog_id = doc.new_object_id();
    doc.objects.insert(
        catalog_id,
        Object::Dictionary(dictionary! {
            "Type" => "Catalog",
            "Pages" => Object::Reference(pages_id),
        }),
    );

    doc.trailer.set("Root", Object::Reference(catalog_id));
    doc.max_id = doc.objects.keys().map(|id| id.0).max().unwrap_or(0);

    // Save
    let mut file = File::create(path)?;
    doc.save_to(&mut file)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    Ok(())
}

/// Generate a multi-page PDF.
pub fn generate_multipage_pdf(path: &Path, pages_content: &[&str]) -> std::io::Result<()> {
    let mut doc = Document::with_version("1.5");

    // Create font
    let font_id = doc.new_object_id();
    doc.objects.insert(
        font_id,
        Object::Dictionary(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
        }),
    );

    // Create resources
    let resources_id = doc.new_object_id();
    doc.objects.insert(
        resources_id,
        Object::Dictionary(dictionary! {
            "Font" => dictionary! {
                "F1" => Object::Reference(font_id),
            },
        }),
    );

    // Create pages node first
    let pages_id = doc.new_object_id();

    // Create each page
    let mut page_refs = Vec::new();
    for content_text in pages_content {
        let content_ops = create_text_operations(content_text, 72.0, 720.0);
        let content = Content {
            operations: content_ops,
        };
        let content_bytes = content.encode().unwrap_or_default();
        let content_stream = Stream::new(dictionary! {}, content_bytes);
        let content_id = doc.add_object(content_stream);

        let page_id = doc.new_object_id();
        doc.objects.insert(
            page_id,
            Object::Dictionary(dictionary! {
                "Type" => "Page",
                "Parent" => Object::Reference(pages_id),
                "MediaBox" => vec![
                    Object::Integer(0),
                    Object::Integer(0),
                    Object::Integer(612),
                    Object::Integer(792),
                ],
                "Contents" => Object::Reference(content_id),
                "Resources" => Object::Reference(resources_id),
            }),
        );
        page_refs.push(Object::Reference(page_id));
    }

    // Create pages node
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => page_refs,
            "Count" => Object::Integer(pages_content.len() as i64),
        }),
    );

    // Create catalog
    let catalog_id = doc.new_object_id();
    doc.objects.insert(
        catalog_id,
        Object::Dictionary(dictionary! {
            "Type" => "Catalog",
            "Pages" => Object::Reference(pages_id),
        }),
    );

    doc.trailer.set("Root", Object::Reference(catalog_id));
    doc.max_id = doc.objects.keys().map(|id| id.0).max().unwrap_or(0);

    // Save
    let mut file = File::create(path)?;
    doc.save_to(&mut file)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    Ok(())
}

/// Create text operations for PDF content.
fn create_text_operations(text: &str, x: f32, y: f32) -> Vec<Operation> {
    let mut ops = Vec::new();

    // Begin text block
    ops.push(Operation::new("BT", vec![]));

    // Set font
    ops.push(Operation::new("Tf", vec!["F1".into(), Object::Integer(12)]));

    // Move to position
    ops.push(Operation::new("Td", vec![Object::Real(x), Object::Real(y)]));

    // Split text by lines and add each
    let lines: Vec<&str> = text.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            // Move to next line
            ops.push(Operation::new(
                "Td",
                vec![Object::Integer(0), Object::Integer(-14)],
            ));
        }
        // Escape text for PDF
        let escaped = line
            .replace('\\', "\\\\")
            .replace('(', "\\(")
            .replace(')', "\\)");
        ops.push(Operation::new("Tj", vec![Object::string_literal(escaped)]));
    }

    // End text block
    ops.push(Operation::new("ET", vec![]));

    ops
}

/// Set up all PDF fixtures.
pub fn setup_fixtures() -> std::io::Result<()> {
    let fixtures_dir = Path::new("tests/fixtures/pdf");
    if !fixtures_dir.exists() {
        std::fs::create_dir_all(fixtures_dir)?;
    }

    // Generate simple PDF
    generate_simple_pdf(
        &fixtures_dir.join("simple.pdf"),
        "This is a simple PDF document.\nIt contains plain text for testing.",
    )?;

    // Generate multipage PDF
    generate_multipage_pdf(
        &fixtures_dir.join("multipage.pdf"),
        &[
            "Page 1: This is the first page of the document.",
            "Page 2: Second page with more content.",
            "Page 3: Third and final page.",
        ],
    )?;

    // Generate PDF with PII
    generate_simple_pdf(
        &fixtures_dir.join("with_pii.pdf"),
        "Personal Information Document\n\n\
         Name: John Doe\n\
         Email: john.doe@example.com\n\
         Phone: 555-123-4567\n\
         SSN: 123-45-6789\n\
         Address: 123 Main Street, City, ST 12345",
    )?;

    // Generate empty PDF
    generate_simple_pdf(&fixtures_dir.join("empty.pdf"), "")?;

    Ok(())
}
