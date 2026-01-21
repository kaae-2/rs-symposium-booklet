use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "symposium-booklet")]
#[command(about = "Build a symposium booklet from Excel to Markdown + Typst", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Build markdown files and optionally render PDFs
    Build(BuildOpts),
    /// Validate input files without writing output
    Validate {
        /// Input workbook or directory
        input: String,
    },
}

#[derive(clap::Args, Clone)]
pub struct BuildOpts {
    /// Input workbook (.xlsx) or directory
    #[arg(long)]
    pub input: String,

    /// Output directory
    #[arg(long)]
    pub output: String,

    /// Template directory override
    #[arg(long)]
    pub template: Option<String>,

    /// Comma separated locales (default en,da)
    #[arg(long, default_value = "en,da")]
    pub locales: String,

    /// Dry run: validate and print actions without writing
    #[arg(long)]
    pub dry_run: bool,

    /// Emit parse JSON into `output/tools_output/parse.json` and exit (no other writes)
    #[arg(long)]
    pub emit_parse_json: bool,

    /// Verbose logging
    #[arg(long)]
    pub verbose: bool,

    /// Path to typst binary
    #[arg(long)]
    pub typst_bin: Option<String>,
}
