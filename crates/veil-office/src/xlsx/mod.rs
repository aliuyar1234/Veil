//! Excel spreadsheet (.xlsx) parsing module.

mod cell_ref;
mod parser;
mod streaming;

pub use cell_ref::CellReference;
pub use parser::{cell_to_string, parse_xlsx};
pub use streaming::{parse_xlsx_large, parse_xlsx_streaming, StreamingConfig, StreamingStats};
