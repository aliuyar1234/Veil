//! Test fixture generator for Office documents.
//!
//! Generates minimal valid DOCX, XLSX, and PPTX files for testing.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use zip::write::FileOptions;
use zip::ZipWriter;

/// Content Types XML (required for all OOXML formats)
#[allow(dead_code)]
const CONTENT_TYPES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
</Types>"#;

/// Root relationships
#[allow(dead_code)]
const RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
</Relationships>"#;

pub fn ensure_fixtures_dir() -> std::io::Result<()> {
    let base = Path::new("tests/fixtures");
    fs::create_dir_all(base.join("docx"))?;
    fs::create_dir_all(base.join("xlsx"))?;
    fs::create_dir_all(base.join("pptx"))?;
    fs::create_dir_all(base.join("legacy"))?;
    Ok(())
}

/// Generate a simple DOCX file with the given text content.
pub fn generate_simple_docx(path: &Path, content: &str) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // [Content_Types].xml
    let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;
    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(content_types.as_bytes())?;

    // _rels/.rels
    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;
    zip.start_file("_rels/.rels", options)?;
    zip.write_all(rels.as_bytes())?;

    // word/document.xml
    let document = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:r>
        <w:t>{}</w:t>
      </w:r>
    </w:p>
  </w:body>
</w:document>"#,
        escape_xml(content)
    );
    zip.start_file("word/document.xml", options)?;
    zip.write_all(document.as_bytes())?;

    // word/_rels/document.xml.rels
    let doc_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
</Relationships>"#;
    zip.start_file("word/_rels/document.xml.rels", options)?;
    zip.write_all(doc_rels.as_bytes())?;

    zip.finish()?;
    Ok(())
}

/// Generate a DOCX file with multiple paragraphs.
pub fn generate_multi_paragraph_docx(path: &Path, paragraphs: &[&str]) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // [Content_Types].xml
    let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;
    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(content_types.as_bytes())?;

    // _rels/.rels
    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;
    zip.start_file("_rels/.rels", options)?;
    zip.write_all(rels.as_bytes())?;

    // word/document.xml with multiple paragraphs
    let para_xml: String = paragraphs
        .iter()
        .map(|p| format!(r#"<w:p><w:r><w:t>{}</w:t></w:r></w:p>"#, escape_xml(p)))
        .collect();

    let document = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    {}
  </w:body>
</w:document>"#,
        para_xml
    );
    zip.start_file("word/document.xml", options)?;
    zip.write_all(document.as_bytes())?;

    // word/_rels/document.xml.rels
    let doc_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
</Relationships>"#;
    zip.start_file("word/_rels/document.xml.rels", options)?;
    zip.write_all(doc_rels.as_bytes())?;

    zip.finish()?;
    Ok(())
}

/// Generate a simple XLSX file with the given cell data.
pub fn generate_simple_xlsx(path: &Path, data: &[Vec<&str>]) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // [Content_Types].xml
    let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
  <Override PartName="/xl/sharedStrings.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"/>
</Types>"#;
    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(content_types.as_bytes())?;

    // _rels/.rels
    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#;
    zip.start_file("_rels/.rels", options)?;
    zip.write_all(rels.as_bytes())?;

    // xl/workbook.xml
    let workbook = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets>
    <sheet name="Sheet1" sheetId="1" r:id="rId1"/>
  </sheets>
</workbook>"#;
    zip.start_file("xl/workbook.xml", options)?;
    zip.write_all(workbook.as_bytes())?;

    // xl/_rels/workbook.xml.rels
    let wb_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings" Target="sharedStrings.xml"/>
</Relationships>"#;
    zip.start_file("xl/_rels/workbook.xml.rels", options)?;
    zip.write_all(wb_rels.as_bytes())?;

    // Collect all strings and create shared strings
    let mut strings: Vec<String> = Vec::new();
    for row in data {
        for cell in row {
            if !strings.contains(&cell.to_string()) {
                strings.push(cell.to_string());
            }
        }
    }

    // xl/sharedStrings.xml
    let shared_strings_items: String = strings
        .iter()
        .map(|s| format!("<si><t>{}</t></si>", escape_xml(s)))
        .collect();
    let shared_strings = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="{}" uniqueCount="{}">
{}
</sst>"#,
        strings.len(),
        strings.len(),
        shared_strings_items
    );
    zip.start_file("xl/sharedStrings.xml", options)?;
    zip.write_all(shared_strings.as_bytes())?;

    // xl/worksheets/sheet1.xml
    let mut rows_xml = String::new();
    for (row_idx, row) in data.iter().enumerate() {
        rows_xml.push_str(&format!(r#"<row r="{}">"#, row_idx + 1));
        for (col_idx, cell) in row.iter().enumerate() {
            let col_letter = column_letter(col_idx);
            let cell_ref = format!("{}{}", col_letter, row_idx + 1);
            let string_idx = strings.iter().position(|s| s == *cell).unwrap_or(0);
            rows_xml.push_str(&format!(
                r#"<c r="{}" t="s"><v>{}</v></c>"#,
                cell_ref, string_idx
            ));
        }
        rows_xml.push_str("</row>");
    }

    let sheet = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>
    {}
  </sheetData>
</worksheet>"#,
        rows_xml
    );
    zip.start_file("xl/worksheets/sheet1.xml", options)?;
    zip.write_all(sheet.as_bytes())?;

    zip.finish()?;
    Ok(())
}

/// Generate a simple PPTX file with the given slide content.
pub fn generate_simple_pptx(path: &Path, slides: &[&str]) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // Build content types with all slides
    let mut slide_overrides = String::new();
    for i in 1..=slides.len() {
        slide_overrides.push_str(&format!(
            r#"<Override PartName="/ppt/slides/slide{}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>"#,
            i
        ));
    }

    let content_types = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  {}
</Types>"#,
        slide_overrides
    );
    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(content_types.as_bytes())?;

    // _rels/.rels
    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>"#;
    zip.start_file("_rels/.rels", options)?;
    zip.write_all(rels.as_bytes())?;

    // ppt/presentation.xml
    let mut slide_ids = String::new();
    for i in 1..=slides.len() {
        slide_ids.push_str(&format!(r#"<p:sldId id="{}" r:id="rId{}"/>"#, 255 + i, i));
    }

    let presentation = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:sldIdLst>
    {}
  </p:sldIdLst>
</p:presentation>"#,
        slide_ids
    );
    zip.start_file("ppt/presentation.xml", options)?;
    zip.write_all(presentation.as_bytes())?;

    // ppt/_rels/presentation.xml.rels
    let mut pres_rels = String::new();
    for i in 1..=slides.len() {
        pres_rels.push_str(&format!(
            r#"<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide{}.xml"/>"#,
            i, i
        ));
    }
    let pres_rels_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  {}
</Relationships>"#,
        pres_rels
    );
    zip.start_file("ppt/_rels/presentation.xml.rels", options)?;
    zip.write_all(pres_rels_xml.as_bytes())?;

    // Create each slide
    for (i, content) in slides.iter().enumerate() {
        let slide = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr>
        <p:cNvPr id="1" name=""/>
        <p:cNvGrpSpPr/>
        <p:nvPr/>
      </p:nvGrpSpPr>
      <p:grpSpPr/>
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="2" name="Title"/>
          <p:cNvSpPr/>
          <p:nvPr/>
        </p:nvSpPr>
        <p:spPr/>
        <p:txBody>
          <a:bodyPr/>
          <a:lstStyle/>
          <a:p>
            <a:r>
              <a:t>{}</a:t>
            </a:r>
          </a:p>
        </p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
</p:sld>"#,
            escape_xml(content)
        );
        zip.start_file(format!("ppt/slides/slide{}.xml", i + 1), options)?;
        zip.write_all(slide.as_bytes())?;
    }

    zip.finish()?;
    Ok(())
}

/// Convert column index to letter (0 -> A, 1 -> B, etc.)
fn column_letter(index: usize) -> String {
    let mut result = String::new();
    let mut n = index;
    loop {
        result.insert(0, (b'A' + (n % 26) as u8) as char);
        if n < 26 {
            break;
        }
        n = n / 26 - 1;
    }
    result
}

/// Escape special XML characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_letter() {
        assert_eq!(column_letter(0), "A");
        assert_eq!(column_letter(1), "B");
        assert_eq!(column_letter(25), "Z");
        assert_eq!(column_letter(26), "AA");
        assert_eq!(column_letter(27), "AB");
    }
}
