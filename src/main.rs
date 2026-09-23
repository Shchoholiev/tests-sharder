use std::{
    env,
    ffi::OsStr,
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
    num::NonZeroU32,
    path::{Path, PathBuf},
    process::ExitCode,
};

use serde::{Deserialize, Serialize};
use tests_sharder::{sharder::shard_tests, test_case::Test};

const USAGE: &str = "Usage: tests-sharder --target-ms <positive integer> [path]";

struct Args {
    target_ms: NonZeroU32,
    path: Option<PathBuf>,
}

#[derive(Deserialize)]
struct InputTest {
    id: String,
    duration_ms: u32,
}

#[derive(Serialize)]
struct OutputTest<'a> {
    id: &'a str,
    duration_ms: u32,
}

#[derive(Serialize)]
struct OutputShard<'a> {
    shard: usize,
    tests: Vec<OutputTest<'a>>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let Some(args) = parse_args()? else {
        println!("{USAGE}");
        return Ok(());
    };

    let reader: Box<dyn BufRead> = match args.path.as_deref() {
        Some(path) if path != Path::new("-") => Box::new(BufReader::new(
            File::open(path).map_err(|error| format!("{}: {error}", path.display()))?,
        )),
        _ => Box::new(BufReader::new(io::stdin())),
    };

    let mut tests = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let line_number = index + 1;
        let line = line.map_err(|error| format!("line {line_number}: {error}"))?;
        let input: InputTest =
            serde_json::from_str(&line).map_err(|error| format!("line {line_number}: {error}"))?;
        if input.duration_ms == 0 {
            return Err(format!("line {line_number}: duration_ms must be positive"));
        }
        tests.push(Test::new(input.id, input.duration_ms));
    }

    let shards = shard_tests(tests, args.target_ms.get());
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    for (shard, tests) in shards.iter().enumerate() {
        let output = OutputShard {
            shard,
            tests: tests
                .iter()
                .map(|test| OutputTest {
                    id: &test.id,
                    duration_ms: test.duration_ms.get(),
                })
                .collect(),
        };
        serde_json::to_writer(&mut writer, &output)
            .map_err(|error| format!("failed to write output: {error}"))?;
        writeln!(writer).map_err(|error| format!("failed to write output: {error}"))?;
    }
    writer
        .flush()
        .map_err(|error| format!("failed to write output: {error}"))
}

fn parse_args() -> Result<Option<Args>, String> {
    let mut target_ms = None;
    let mut path = None;
    let mut args = env::args_os().skip(1);

    while let Some(arg) = args.next() {
        if arg == OsStr::new("--help") || arg == OsStr::new("-h") {
            return Ok(None);
        }
        if arg == OsStr::new("--target-ms") {
            if target_ms.is_some() {
                return Err(format!("--target-ms may be provided only once\n{USAGE}"));
            }
            let raw = args
                .next()
                .ok_or_else(|| format!("missing value for --target-ms\n{USAGE}"))?;
            let value = raw
                .to_str()
                .and_then(|value| value.parse::<u32>().ok())
                .and_then(NonZeroU32::new)
                .ok_or_else(|| format!("--target-ms must be a positive u32\n{USAGE}"))?;
            target_ms = Some(value);
        } else if arg != OsStr::new("-") && arg.to_string_lossy().starts_with('-') {
            return Err(format!(
                "unknown option: {}\n{USAGE}",
                arg.to_string_lossy()
            ));
        } else if path.is_some() {
            return Err(format!("only one input path is allowed\n{USAGE}"));
        } else {
            path = Some(PathBuf::from(arg));
        }
    }

    let target_ms = target_ms.ok_or_else(|| format!("missing --target-ms\n{USAGE}"))?;
    Ok(Some(Args { target_ms, path }))
}
