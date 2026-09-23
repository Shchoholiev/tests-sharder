use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
    num::NonZeroU32,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Parser;
use serde::{Deserialize, Serialize};
use tests_sharder::{sharder::shard_tests, test_case::Test};

#[derive(Parser)]
#[command(about = "Split JSONL tests into time-balanced shards")]
struct Args {
    #[arg(
        long,
        value_parser = parse_target_ms,
        help = "Target shard duration in milliseconds"
    )]
    target_ms: NonZeroU32,
    #[arg(
        allow_hyphen_values = true,
        default_value = "-",
        help = "JSONL file; reads stdin if omitted or '-'"
    )]
    path: PathBuf,
}

#[derive(Deserialize)]
struct InputTest {
    id: String,
    duration_ms: u32,
}

#[derive(Serialize)]
struct Shard {
    shard: usize,
    tests: Vec<Test>,
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Args) -> Result<(), String> {
    let tests = load_tests(&args.path)?;
    let shards = shard_tests(tests, args.target_ms.get());
    write_shards(shards)
}

fn load_tests(path: &Path) -> Result<Vec<Test>, String> {
    if path == Path::new("-") {
        return parse_jsonl(io::stdin().lock());
    }
    let file = File::open(path)
        .map_err(|error| format!("cannot open input file '{}': {error}", path.display()))?;
    parse_jsonl(BufReader::new(file))
}

fn parse_jsonl(reader: impl BufRead) -> Result<Vec<Test>, String> {
    let mut tests = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let line_number = index + 1;
        let line = line.map_err(|error| format!("line {line_number}: {error}"))?;
        let input: InputTest = serde_json::from_str(&line)
            .map_err(|error| format!("line {line_number}: invalid JSONL test: {error}"))?;
        let duration_ms = NonZeroU32::new(input.duration_ms)
            .ok_or_else(|| format!("line {line_number}: duration_ms must be positive"))?;
        tests.push(Test {
            id: input.id,
            duration_ms,
        });
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

fn parse_target_ms(value: &str) -> Result<NonZeroU32, String> {
    value
        .parse()
        .map_err(|_| "must be a positive integer".to_owned())
}
