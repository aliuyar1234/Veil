//! Context-aware detection module.
//!
//! This module provides context analysis to improve PII detection accuracy
//! by examining surrounding text for contextual markers like honorifics,
//! labels, and suppression patterns.
//!
//! # Features
//!
//! - **Marker Detection**: Detect honorifics (Mr., Dr.), labels (Name:, Email:), and suppression patterns
//! - **Address Detection**: Detect multi-line postal addresses in EN/DE/FR formats
//! - **Table Context**: Use CSV/table column headers to boost PII detection confidence
//! - **Custom Rules**: Load user-defined context rules from YAML configuration files
//!
//! # Example
//!
//! ```rust,ignore
//! use veil_detect::context::{ContextAnalyzer, ConfigLoader};
//!
//! // Create analyzer with built-in rules
//! let mut analyzer = ContextAnalyzer::new();
//!
//! // Optionally load custom rules
//! let custom_rules = ConfigLoader::from_file("my_rules.yaml")?;
//! analyzer.add_rules(custom_rules);
//!
//! // Analyze text
//! let text = "Contact: Mr. John Smith";
//! let finding = /* ... detected finding for "John Smith" */;
//!
//! let analysis = analyzer.analyze_finding(&finding, text, Some("en"));
//! println!("Adjusted confidence: {}", analysis.adjusted_confidence);
//! ```

pub mod address;
pub mod analyzer;
pub mod builtin;
mod error;
pub mod loader;
pub mod marker;
pub mod rule;
pub mod table;

pub use address::{AddressBlock, AddressComponent, AddressComponentType, AddressDetector};
pub use analyzer::{ContextAnalysis, ContextAnalyzer};
pub use error::ContextError;
pub use loader::{ConfigLoader, ContextConfig, RuleConfig};
pub use marker::{ContextMarker, MarkerType};
pub use rule::{ContextAction, ContextRule};
pub use table::{TableContext, TableDetector, TableHeader};
