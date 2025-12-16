//! Excel cell reference utilities.

/// A reference to a cell in an Excel spreadsheet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellReference {
    /// Sheet name.
    pub sheet: String,
    /// 1-based row number.
    pub row: u32,
    /// 0-based column index.
    pub column: u32,
}

impl CellReference {
    /// Create a new cell reference.
    pub fn new(sheet: impl Into<String>, row: u32, column: u32) -> Self {
        Self {
            sheet: sheet.into(),
            row,
            column,
        }
    }

    /// Convert a 0-based column index to Excel letter notation.
    ///
    /// # Examples
    ///
    /// - 0 -> "A"
    /// - 25 -> "Z"
    /// - 26 -> "AA"
    /// - 27 -> "AB"
    /// - 702 -> "AAA"
    pub fn column_letter(column: u32) -> String {
        let mut result = String::new();
        let mut col = column;

        loop {
            let remainder = col % 26;
            result.insert(0, (b'A' + remainder as u8) as char);

            if col < 26 {
                break;
            }
            col = col / 26 - 1;
        }

        result
    }

    /// Get the column letter for this cell reference.
    pub fn column_as_letter(&self) -> String {
        Self::column_letter(self.column)
    }

    /// Format as a full cell reference string (e.g., "Sheet1!A1").
    pub fn to_full_reference(&self) -> String {
        format!(
            "{}!{}{}",
            self.sheet,
            Self::column_letter(self.column),
            self.row
        )
    }

    /// Format as a cell reference without sheet name (e.g., "A1").
    pub fn to_cell_only(&self) -> String {
        format!("{}{}", Self::column_letter(self.column), self.row)
    }
}

impl std::fmt::Display for CellReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_full_reference())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_letter_single() {
        assert_eq!(CellReference::column_letter(0), "A");
        assert_eq!(CellReference::column_letter(1), "B");
        assert_eq!(CellReference::column_letter(25), "Z");
    }

    #[test]
    fn test_column_letter_double() {
        assert_eq!(CellReference::column_letter(26), "AA");
        assert_eq!(CellReference::column_letter(27), "AB");
        assert_eq!(CellReference::column_letter(51), "AZ");
        assert_eq!(CellReference::column_letter(52), "BA");
        assert_eq!(CellReference::column_letter(701), "ZZ");
    }

    #[test]
    fn test_column_letter_triple() {
        assert_eq!(CellReference::column_letter(702), "AAA");
        assert_eq!(CellReference::column_letter(703), "AAB");
    }

    #[test]
    fn test_cell_reference_display() {
        let cell = CellReference::new("Sheet1", 5, 1);
        assert_eq!(cell.to_string(), "Sheet1!B5");
        assert_eq!(cell.to_cell_only(), "B5");
    }

    #[test]
    fn test_cell_reference_with_spaces() {
        let cell = CellReference::new("My Sheet", 10, 26);
        assert_eq!(cell.to_string(), "My Sheet!AA10");
    }
}
