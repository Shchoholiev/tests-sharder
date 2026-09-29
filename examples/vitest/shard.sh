#!/usr/bin/env bash
set -euo pipefail

case "${1:-}" in
  plan)
    "${TESTS_SHARDER_BIN:-../../target/debug/tests-sharder}" \
      --target-ms "${3:?target milliseconds required}" \
      "${2:?timings JSONL required}" > "${4:?plan path required}"
    cat "$4"
    ;;
  run)
    plan="${2:?plan path required}"
    index="${3:?shard index required}"
    count="$(wc -l < "$plan" | tr -d '[:space:]')"
    shift 3
    SHARD_PLAN="$(cd "$(dirname "$plan")" && pwd)/$(basename "$plan")" \
      ./node_modules/.bin/vitest run --shard="$index/$count" "$@"
    ;;
  *)
    echo 'usage: shard.sh plan <timings> <target-ms> <plan> | run <plan> <index> [vitest options]' >&2
    exit 1
    ;;
esac
