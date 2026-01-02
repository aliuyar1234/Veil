use veil_types::{ParseWarning, WarningCode};

pub(crate) fn xml_parse_warning(source: &str) -> ParseWarning {
    ParseWarning {
        code: WarningCode::XmlParseError,
        message: format!(
            "Failed to fully parse XML from `{source}`; extracted content may be incomplete."
        ),
        position: None,
    }
}
