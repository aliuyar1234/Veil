//! Veil CLI - PII detection and redaction tool.

mod cli;
mod commands;
mod output;

use clap::Parser;
use miette::Result;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan(args) => commands::scan::run(args, cli.quiet, cli.json),
        Commands::Protect(args) => commands::protect::run(args, cli.quiet, cli.json),
        Commands::Batch(args) => commands::batch::run(args, cli.quiet, cli.json),
        Commands::Discover(args) => commands::discover::run(args, cli.quiet, cli.json),
        Commands::Crypto(args) => commands::crypto::run(args, cli.quiet, cli.json),
        Commands::Stream(args) => commands::stream::run(args, cli.quiet, cli.json),
        Commands::Policy(args) => commands::policy::run(args),
    }
}
