#![doc = include_str!("../README.md")]

use anyhow::Result;
use clap::StructOpt;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

use self::args::{Args, Command};

mod args;

const SEPERATOR_CHARS: &str = r#"*.,{}()[]:;?'"<>\/=+-@!|#&%$"#;

fn main() -> Result<()> {
    let args = Args::parse();

    let words_to_ignore = args
        .ignored_words_files
        .map_or_else(|| Ok(HashSet::default()), read_words_to_ignore)?;
    let word_counts = count_words(
        args.files,
        Some(|word: &str| {
            // word must be long enough
            word.len() >= args.minimum_word_length
            // word cannot be a number
            && word.parse::<u128>().is_err()
            // word must not be ignored
            && !words_to_ignore.contains(word)
        }),
    )?;
    let (sum, statistics) = calculate_statistics(word_counts, args.top);

    match args.command {
        Command::Stats => {
            println!("Total words: {sum}");
            for (word, count, frequency) in statistics {
                println!("{word}\t{count}\t{frequency:.2}");
            }
        }
        Command::Words => {
            for (word, count, _) in statistics {
                print!("{}", format!("{word}\n").repeat(count));
            }
        }
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

fn count_words<F: Fn(&str) -> bool>(
    files: impl IntoIterator<Item = PathBuf>,
    filter: Option<F>,
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
        .filter(|word| filter.as_ref().map_or(true, |f| f(word)))
        .fold(HashMap::new(), |mut map, word| {
            map.entry(word).and_modify(|count| *count += 1).or_insert(1);
            map
        });
    Ok(word_counts)
}

fn read_words_to_ignore(
    ignored_words_files: impl IntoIterator<Item = PathBuf>,
) -> Result<HashSet<String>, std::io::Error> {
    let ignore_files: Result<Vec<_>, _> = ignored_words_files
        .into_iter()
        .map(fs::read_to_string)
        .collect();
    Ok(ignore_files?
        .iter()
        .flat_map(|content| content.lines())
        .map(|word| word.strip_prefix(char::is_whitespace).unwrap_or(word))
        .map(|word| word.strip_suffix(char::is_whitespace).unwrap_or(word))
        .filter(|word| !word.is_empty())
        .map(String::from)
        .collect())
}
