use std::path::PathBuf;

use clap::{ArgAction, Parser};

/// A CLI tool to translate images.
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Input file or directory
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output directory
    #[arg(short, long)]
    pub output: PathBuf,

    /// Optional config file
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Verbose mode (-v, -vv, -vvv)
    #[arg(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    /// Overwrite already translated images
    #[arg(long)]
    pub overwrite: bool,

    /// maximum batch size for ocr
    #[arg(long, default_value_t = 16)]
    pub max_batch_size_ocr: usize,

    /// maximum batch size for upscaler
    #[arg(long, default_value_t = 2)]
    pub max_batch_size_upscaler: usize,
}
