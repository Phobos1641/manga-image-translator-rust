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

    /// uses cuda for translator
    #[arg(long)]
    pub cuda: bool,

    /// maximum batch size for cuda
    #[arg(long, default_value_t = 2)]
    pub max_batch_size: usize,
}
