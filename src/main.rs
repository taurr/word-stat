#![doc = include_str!("../README.md")]

use anyhow::Result;
use clap::StructOpt;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

mod args;

const SEPERATOR_CHARS: &str = "*.,{}()[]:;?'\"<>\\/=+-@!|#&%$";

fn main() -> Result<()> {
    let args = args::Args::parse();

    let ignored_words = match args.ignored_words_files {
        Some(ignored_words_files) => read_ignored_words(&ignored_words_files)?,
        None => HashSet::default(),
    };
    let word_counts = get_word_counts(args.files, args.minimum_word_length, ignored_words)?;
    let (sum, statistics) = calculate_statistics(word_counts, args.top);

    match args.command {
        args::Command::Stats => {
            println!("Total words: {sum}");
            for (word, count, frequency) in statistics {
                println!("{word}\t{count}\t{frequency:.2}");
            }
        }
        args::Command::Words => {
            for (word, count, _) in statistics {
                print!("{}", format!("{word}\n").repeat(count));
            }
        },
    }

    Ok(())
}

fn calculate_statistics(
    word_counts: HashMap<String, usize>,
    number_of_results: Option<usize>,
) -> (usize, Vec<(String, usize, f64)>) {
    let mut word_counts: Vec<_> = word_counts.into_iter().collect();
    word_counts.sort_by(|a, b| b.1.cmp(&a.1));
    let number_of_results = number_of_results.unwrap_or(word_counts.len());
    word_counts = word_counts.into_iter().take(number_of_results).collect();
    let sum: usize = word_counts.iter().map(|(_word, count)| count).sum();
    let statistics = word_counts
        .into_iter()
        .map(|(word, count)| (word, count, count as f64 / sum as f64 * 100.0))
        .collect();
    (sum, statistics)
}

fn get_word_counts(
    files: Vec<PathBuf>,
    minimum_word_length: usize,
    ignored_words: HashSet<String>,
) -> Result<HashMap<String, usize>, anyhow::Error> {
    let file_contents: Result<Vec<_>, _> = files.into_iter().map(fs::read_to_string).collect();
    let word_counts = file_contents?
        .into_iter()
        .flat_map(|file_content| {
            file_content
                .split(|ch: char| ch.is_whitespace() || SEPERATOR_CHARS.contains(ch))
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .filter(|word| {
            word.len() >= minimum_word_length
                && word.parse::<u128>().is_err()
                && !ignored_words.contains(word)
        })
        .fold(HashMap::new(), |mut map, word| {
            map.entry(word).and_modify(|count| *count += 1).or_insert(1);
            map
        });
    Ok(word_counts)
}

fn read_ignored_words(ignored_words_files: &[PathBuf]) -> Result<HashSet<String>, std::io::Error> {
    let ignore_files: Result<Vec<_>, _> =
        ignored_words_files.iter().map(fs::read_to_string).collect();
    Ok(ignore_files?
        .iter()
        .flat_map(|content| content.lines())
        .map(|word| word.strip_prefix(char::is_whitespace).unwrap_or(word))
        .map(|word| word.strip_suffix(char::is_whitespace).unwrap_or(word))
        .filter(|word| !word.is_empty())
        .map(String::from)
        .collect())
}
