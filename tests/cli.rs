use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Output, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;

const INPUT: &str = r#"{"id":"a","duration_ms":3}
{"id":"b","duration_ms":2}
{"id":"c","duration_ms":3}
{"id":"d","duration_ms":2}
{"id":"e","duration_ms":2}
"#;

#[test]
fn piped_input_produces_valid_jsonl_with_each_test_once() {
    let output = run(&["--target-ms", "6"], INPUT);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    let shards: Vec<Value> = stdout
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(shards.len(), 2);

    let mut actual = BTreeMap::new();
    for (index, shard) in shards.iter().enumerate() {
        assert_eq!(shard["shard"], index);
        let tests = shard["tests"].as_array().unwrap();
        let total: u64 = tests
            .iter()
            .map(|test| {
                let id = test["id"].as_str().unwrap();
                let duration = test["duration_ms"].as_u64().unwrap();
                assert!(actual.insert(id.to_owned(), duration).is_none());
                duration
            })
            .sum();
        assert!(total <= 6);
    }
    assert_eq!(
        actual,
        BTreeMap::from([
            ("a".to_owned(), 3),
            ("b".to_owned(), 2),
            ("c".to_owned(), 3),
            ("d".to_owned(), 2),
            ("e".to_owned(), 2),
        ])
    );
}

#[test]
fn file_and_explicit_stdin_produce_the_same_output() {
    let path = temporary_path();
    fs::write(&path, INPUT).unwrap();
    let file_output = run(&["--target-ms", "6", path.to_str().unwrap()], "");
    fs::remove_file(path).unwrap();
    let stdin_output = run(&["--target-ms", "6", "-"], INPUT);

    assert!(file_output.status.success());
    assert!(stdin_output.status.success());
    assert_eq!(file_output.stdout, stdin_output.stdout);
}

#[test]
fn empty_input_produces_no_shards() {
    let output = run(&["--target-ms", "6"], "");
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn invalid_target_reports_error() {
    assert_error(&["--target-ms", "0"], "", "--target-ms");
}

#[test]
fn malformed_json_reports_line_number() {
    assert_error(&["--target-ms", "6"], "not json\n", "line 1");
}

#[test]
fn zero_duration_reports_error() {
    assert_error(
        &["--target-ms", "6"],
        r#"{"id":"a","duration_ms":0}"#,
        "duration_ms must be positive",
    );
}

#[test]
fn missing_file_reports_error() {
    let missing_path = temporary_path();
    let path = missing_path.to_str().unwrap();
    assert_error(&["--target-ms", "6", path], "", path);
}

fn run(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tests-sharder"))
        .args(args)
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

fn assert_error(args: &[&str], input: &str, expected_error: &str) {
    let output = run(args, input);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains(expected_error));
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
