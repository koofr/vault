// Usage:
// cargo run --package vault-intl-extract -- --include '**/*.rs' --exclude 'vault-intl/**/*.rs' --exclude 'target/**/*' --out-file 'vault-core/src/intl/locales/en/extracted.json'

use std::path::PathBuf;

use clap::Parser;
use glob::glob;
use indexmap::IndexMap;

use vault_intl_extract::{FormatMessageExtractor, Message};

#[derive(Parser, Debug)]
#[command(name = "intl_extract")]
#[command(about = "Extract format_message! macro calls into FormatJS JSON", long_about = None)]
struct Args {
    /// Glob patterns for Rust source files to include (e.g., "src/**/*.rs")
    #[arg(long, num_args = 1..)]
    include: Vec<String>,

    /// Glob patterns for Rust source files to exclude (e.g., "tests/**/*.rs")
    #[arg(long, num_args = 1..)]
    exclude: Vec<String>,

    /// Output JSON file
    #[arg(short, long)]
    out_file: PathBuf,
}

fn main() {
    let args = Args::parse();

    let mut extractor = FormatMessageExtractor::new(Box::new(
        |message_id: String, existing_message: String, new_message: String| {
            eprintln!(
                "Duplicate message id: '{}', existing: '{}', new: '{}'",
                message_id, existing_message, new_message
            );
        },
    ));

    // Pre-compile exclude patterns
    let exclude_patterns: Vec<glob::Pattern> = args
        .exclude
        .iter()
        .map(|pattern| glob::Pattern::new(pattern).unwrap())
        .collect();

    // Build a list of included files, excluding any that match an exclude
    // pattern
    let files: Vec<PathBuf> = args
        .include
        .iter()
        .map(|pattern| glob(pattern).unwrap())
        .flatten()
        .map(|entry| entry.unwrap())
        .filter(|path| {
            !exclude_patterns
                .iter()
                .any(|exclude_pat| exclude_pat.matches_path(path))
        })
        .collect();

    // Process filtered files
    for path in &files {
        println!("{:?}", path);
        let content = std::fs::read_to_string(path).unwrap();
        extractor.extract_file(&content).unwrap();
    }

    let mut messages: Vec<(String, Message)> = extractor.messages.into_iter().collect();
    messages.sort_by(|(a, _), (b, _)| a.cmp(b));
    let messages: IndexMap<String, Message> = messages.into_iter().collect();

    std::fs::write(
        args.out_file,
        serde_json::to_string_pretty(&messages).unwrap(),
    )
    .unwrap();

    println!(
        "Extracted {} messages from {} files.",
        messages.len(),
        files.len()
    );
}
