use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Output, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Deserialize;
use tests_sharder::test_case::Test;

#[derive(Deserialize)]
struct Shard {
    shard: usize,
    tests: Vec<Test>,
}

const INPUT: &str = r#"{"id":"a","duration_ms":3}
{"id":"b","duration_ms":2}
{"id":"c","duration_ms":3}
{"id":"d","duration_ms":2}
{"id":"e","duration_ms":2}
"#;

#[test]
fn piped_input_produces_valid_jsonl_with_each_test_once() {
    // Arrange
    let input = INPUT;

    // Act
    let output = run_cli_binary("6", None, input);

    // Assert
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let shards = parse_shards(&output.stdout);
    assert_eq!(shards.len(), 2);
    assert_shards_preserve_tests(&shards, input, 6);
}

#[test]
fn file_and_explicit_stdin_produce_the_same_output() {
    // Arrange
    let path = temporary_path();
    fs::write(&path, INPUT).unwrap();

    // Act
    let file_output = run_cli_binary("6", Some(path.to_str().unwrap()), "");
    let stdin_output = run_cli_binary("6", Some("-"), INPUT);

    // Assert
    assert!(file_output.status.success());
    assert!(stdin_output.status.success());
    assert_eq!(file_output.stdout, stdin_output.stdout);

    // Cleanup
    fs::remove_file(path).unwrap();
}

#[test]
fn empty_input_produces_no_shards() {
    // Arrange
    let input = "";

    // Act
    let output = run_cli_binary("6", None, input);

    // Assert
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn invalid_target_reports_error() {
    // Arrange
    let target_ms = "0";

    // Act
    let output = run_cli_binary(target_ms, None, "");

    // Assert
    assert_error(
        &output,
        "invalid value '0' for '--target-ms <TARGET_MS>': must be a positive integer",
    );
}

#[test]
fn malformed_json_reports_line_number() {
    // Arrange
    let input = "not json\n";

    // Act
    let output = run_cli_binary("6", None, input);

    // Assert
    assert_error(&output, "line 1");
}

#[test]
fn zero_duration_reports_error() {
    // Arrange
    let input = r#"{"id":"a","duration_ms":0}"#;

    // Act
    let output = run_cli_binary("6", None, input);

    // Assert
    assert_error(&output, "line 1");
}

#[test]
fn missing_file_reports_error() {
    // Arrange
    let missing_path = temporary_path();
    let path = missing_path.to_str().unwrap();

    // Act
    let output = run_cli_binary("6", Some(path), "");

    // Assert
    assert_error(&output, path);
}

fn run_cli_binary(target_ms: &str, path: Option<&str>, input: &str) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_tests-sharder"));
    command.arg("--target-ms").arg(target_ms);
    if let Some(path) = path {
        command.arg(path);
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("binary should start");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .expect("stdin should be writable");
    child.wait_with_output().expect("binary should exit")
}

fn assert_error(output: &Output, expected_error: &str) {
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected_error),
        "unexpected stderr: {stderr}"
    );
}

fn temporary_path() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "tests-sharder-{}-{nonce}.jsonl",
        std::process::id()
    ))
}

fn parse_shards(output: &[u8]) -> Vec<Shard> {
    let output = std::str::from_utf8(output).unwrap();
    output
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn assert_shards_preserve_tests(shards: &[Shard], input: &str, target_ms: u32) {
    let mut actual = Vec::new();
    for (index, shard) in shards.iter().enumerate() {
        assert_eq!(shard.shard, index);
        let duration_ms: u32 = shard.tests.iter().map(|test| test.duration_ms.get()).sum();
        assert!(duration_ms <= target_ms);
        actual.extend(shard.tests.iter().cloned());
    }

    let mut expected: Vec<Test> = input
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    actual.sort();
    expected.sort();
    assert_eq!(actual, expected);
}
