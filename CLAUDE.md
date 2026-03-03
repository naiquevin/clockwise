# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## About

Clockwise is a Rust CLI tool that reads Emacs org-mode files and aggregates CLOCK entries, displaying total time spent within a specified date range with optional breakdown by day, week, month, quarter, or year.

## Commands

```bash
cargo build                # Debug build
cargo build --release      # Release build
cargo run -- <file.org>    # Run with arguments
cargo test                 # Run all tests
cargo test <test_name>     # Run a single test by name
cargo fmt                  # Format code
cargo fmt --check          # Check formatting without applying
cargo clippy               # Lint
```

## Architecture

**Data flow**: CLI args → parse time duration string into `DateTimeRange` → parse org file CLOCK entries → filter entries by range → aggregate into breakdown buckets (if requested) → print

**Modules**:
- `cli.rs` — clap-based argument parsing and top-level orchestration
- `time_duration.rs` — the largest module; parses the `-t` time
  duration option into a `DateTimeRange`. Handles relative formats
  (`d`, `w-3`, `m`, `q1`, weekday/month names), absolute dates
  (`2026-02-05`, `2026-02`), and ranges (`2025-07-01..2025-07-20`)
- `org_parser.rs` — parses org-mode CLOCK entries from LOGBOOK
  drawers; partitions multi-day entries by day
- `breakdown.rs` — the `-b` option; generates labeled time buckets for
  aggregation
- `datetime_util.rs` — helpers for quarter calculation and
  start-of-period arithmetic
- `error.rs` — `thiserror`-based error enum

**Key type**: `DateTimeRange` (wraps `NaiveDateTime` start/end) is reused throughout — for the time duration CLI option, for parsed clock entries, and for breakdown buckets.

## Developer documentation
- Can be found in the `dev-docs` dir
- Mainly for human use but coding assistants can and should refer to
  it. Prefer reading only the minimum number of docs required to
  perform the task

## Code Guidelines

- Use `NaiveDateTime`/`NaiveDate`/`NaiveTime` (no timezone) — org-mode timestamps are local time
- Refactor into a new module only when a piece of code grows to have significant complexity; don't preemptively split
- Prefer `assert!` / `assert_eq!` in production code when there's no sensible way to handle an error
- Tests live inline in each module under `#[cfg(test)]`
