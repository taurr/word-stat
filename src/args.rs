use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,

    /// Limit the number of results.
    #[clap(global = true, short, long)]
    pub top: Option<usize>,

    /// Minimum word size to consider.
    #[clap(global = true, name("min_len"), long, default_value("3"))]
    pub minimum_word_length: usize,

    /// File with words to ignore. 1 word per line.
    #[clap(global = true, name = "ignore", short = 'i', long = "ignore")]
    pub ignored_words_files: Option<Vec<PathBuf>>,

    /// Files to analyze.
    #[clap(global = true, name = "file")]
    pub files: Vec<PathBuf>,
}

#[derive(clap::Subcommand)]
pub enum Command {
    /// Show word statistics.
    Stats,
    /// Generate a list of words for upload to e.g. 'https://monkeylearn.com/word-cloud'
    Words,
}
