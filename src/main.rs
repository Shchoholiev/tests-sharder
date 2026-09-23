use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
    num::NonZeroU32,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Parser;
use serde::Serialize;
use tests_sharder::{sharder::shard_tests, test_case::Test};

#[derive(Parser)]
#[command(about = "Split JSONL tests into time-balanced shards")]
struct Args {
    #[arg(long, help = "Target shard duration in milliseconds")]
    target_ms: NonZeroU32,
    #[arg(
        allow_hyphen_values = true,
        help = "JSONL file; reads stdin if omitted or '-'"
    )]
    path: Option<PathBuf>,
}

#[derive(Serialize)]
struct Shard {
    shard: usize,
    tests: Vec<Test>,
}

fn main() -> ExitCode {
    let args = Args::parse();
    if let Err(error) = run(args.target_ms, args.path.as_deref()) {
        eprintln!("{error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run(target_ms: NonZeroU32, path: Option<&Path>) -> Result<(), String> {
    let tests = read_tests(open_input(path)?)?;
    let shards = shard_tests(tests, target_ms.get());
    write_shards(shards)
}

fn open_input(path: Option<&Path>) -> Result<Box<dyn BufRead>, String> {
    match path {
        Some(path) if path != Path::new("-") => {
            let file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
            Ok(Box::new(BufReader::new(file)))
        }
        _ => Ok(Box::new(BufReader::new(io::stdin()))),
    }
}

fn read_tests(reader: impl BufRead) -> Result<Vec<Test>, String> {
    let mut tests = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let line_number = index + 1;
        let line = line.map_err(|error| format!("line {line_number}: {error}"))?;
        let test =
            serde_json::from_str(&line).map_err(|error| format!("line {line_number}: {error}"))?;
        tests.push(test);
    }
    Ok(tests)
}

fn write_shards(shards: Vec<Vec<Test>>) -> Result<(), String> {
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    for (shard, tests) in shards.into_iter().enumerate() {
        serde_json::to_writer(&mut writer, &Shard { shard, tests })
            .map_err(|error| format!("stdout: {error}"))?;
        writeln!(writer).map_err(|error| format!("stdout: {error}"))?;
    }
    writer.flush().map_err(|error| format!("stdout: {error}"))
}
