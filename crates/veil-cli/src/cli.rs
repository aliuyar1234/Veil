//! CLI argument definitions.

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

/// Veil - PII detection and redaction tool
#[derive(Parser)]
#[command(name = "veil")]
#[command(
    version,
    about = "Detect and redact personally identifiable information"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Suppress progress output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Output in JSON format
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scan files for PII
    Scan(ScanArgs),
    /// Protect files by redacting PII
    Protect(ProtectArgs),
    /// Batch process multiple files
    Batch(BatchArgs),
    /// Discover PII in a directory tree
    Discover(DiscoverArgs),
    /// Encrypt or decrypt sensitive data
    Crypto(CryptoArgs),
    /// Process streaming input
    Stream(StreamArgs),
    /// Policy management
    Policy(PolicyArgs),
}

/// Arguments for the scan command
#[derive(Parser, Debug)]
pub struct ScanArgs {
    /// Files or directories to scan
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,

    /// Scan directories recursively
    #[arg(short, long)]
    pub recursive: bool,

    /// Policy file to use
    #[arg(short, long)]
    pub policy: Option<PathBuf>,

    /// Limit detection to specific types (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub detect: Option<Vec<String>>,

    /// Exit with code 2 if findings are detected
    #[arg(long)]
    pub fail_on_findings: bool,
}

/// Arguments for the protect command
#[derive(Parser, Debug)]
pub struct ProtectArgs {
    /// Input file
    #[arg(required = true)]
    pub input: PathBuf,

    /// Output file (default: stdout)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Policy file to use
    #[arg(short, long)]
    pub policy: Option<PathBuf>,

    /// Redaction style (label, bar, mask)
    #[arg(long, default_value = "label")]
    pub style: String,
}

/// Arguments for the batch command
#[derive(Parser, Debug)]
pub struct BatchArgs {
    /// Source directories or files to process
    #[arg(required = true)]
    pub sources: Vec<PathBuf>,

    /// Output directory for results
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Number of parallel workers
    #[arg(short = 'j', long, default_value = "4")]
    pub jobs: usize,

    /// Include glob patterns (e.g., "*.txt")
    #[arg(long)]
    pub include: Option<Vec<String>>,

    /// Exclude glob patterns
    #[arg(long)]
    pub exclude: Option<Vec<String>>,

    /// Process ZIP archives
    #[arg(long)]
    pub zip: bool,

    /// Password for encrypted archives
    #[arg(long)]
    pub zip_password: Option<String>,

    /// Maximum file size to process (in MB)
    #[arg(long, default_value = "100")]
    pub max_size: u64,
}

/// Arguments for the discover command
#[derive(Parser, Debug)]
pub struct DiscoverArgs {
    /// Root directory to scan
    #[arg(required = true)]
    pub path: PathBuf,

    /// Output report file
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Report format
    #[arg(long, default_value = "summary", value_enum)]
    pub format: ReportFormatArg,

    /// Maximum files to sample per directory
    #[arg(long)]
    pub sample: Option<usize>,

    /// Include glob patterns
    #[arg(long)]
    pub include: Option<Vec<String>>,

    /// Exclude glob patterns
    #[arg(long)]
    pub exclude: Option<Vec<String>>,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ReportFormatArg {
    Summary,
    Text,
    Json,
}

/// Arguments for the crypto command
#[derive(Parser, Debug)]
pub struct CryptoArgs {
    #[command(subcommand)]
    pub action: CryptoAction,
}

#[derive(Subcommand, Debug)]
pub enum CryptoAction {
    /// Encrypt a file or text
    Encrypt {
        /// Input file (or use --text)
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Text to encrypt
        #[arg(short, long)]
        text: Option<String>,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Encryption key file
        #[arg(short, long)]
        key: PathBuf,
    },
    /// Decrypt a file or text
    Decrypt {
        /// Input file (or use --text)
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Encrypted text (base64)
        #[arg(short, long)]
        text: Option<String>,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Encryption key file
        #[arg(short, long)]
        key: PathBuf,
    },
    /// Generate a new encryption key
    Keygen {
        /// Output key file
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Hash sensitive data
    Hash {
        /// Input file or text
        input: String,

        /// Hash algorithm
        #[arg(long, default_value = "sha256")]
        algorithm: String,
    },
}

/// Arguments for the stream command
#[derive(Parser, Debug)]
pub struct StreamArgs {
    /// Output format for findings
    #[arg(long, default_value = "text")]
    pub format: String,

    /// Chunk size in bytes
    #[arg(long, default_value = "65536")]
    pub chunk_size: usize,
}

/// Arguments for the policy command
#[derive(Parser, Debug)]
pub struct PolicyArgs {
    #[command(subcommand)]
    pub action: PolicyAction,
}

#[derive(Subcommand, Debug)]
pub enum PolicyAction {
    /// Validate a policy file
    Validate {
        /// Policy file to validate
        path: PathBuf,
    },
    /// Generate a sample policy file
    Init {
        /// Output file
        #[arg(short, long, default_value = "veil-policy.yaml")]
        output: PathBuf,
    },
}
