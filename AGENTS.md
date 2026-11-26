# tokenator agent Instructions

Single-crate Rust CLI (edition 2024, MSRV 1.95) that counts LLM prompt tokens
using Hugging Face tokenisers.

## Developer Commands

- `cargo test` — unit + integration. Integration tests
  (`tests/integration/ui.rs`) spawn `./target/debug/tokenator` via rexpect and
  download HF tokenizer files (network required, may be slow/flaky).
- `cargo test --lib` — unit tests only, no network.
- `cargo clippy -- -D warnings` — CI-enforced lints.
- `cargo fmt --check` — edition 2024, Unix newlines.
- `cargo build` then `./target/debug/tokenator <model> '<prompt>'`
  or `--file <path>`.
- `just integration-tests` — `cargo test --test '*'`.
- `just trycmd-snapshot-overwrite` / `just trycmd-snapshot-dump` — manage trycmd
  snapshots (`tests/cmd/*.toml`; `dump/` are gitignored).
- `just insta-snapshot-clean` / `just insta-snapshot-review` — insta snapshot
  management (`cargo insta test`).
- `just coverage` — grcov coverage (needs recent LLVM).

## Architecture & Gotchas

- **Entrypoint:** `src/main.rs` `#[tokio::main] async fn main()`. CLI
  parsing in `src/cli.rs` (`model`, `file`, `prompt`, `--verbose`).
- **Models:** `data/models.json` maps names to `<owner>/<name>` HF repo ids;
  `src/models.rs:get_repo_id` enforces the `<owner>/<name>` pattern. Tokeniser
  JSON is downloaded from HF and cached by hf-hub.
- **Env:** set `HUGGING_FACE_ACCESS_TOKEN` (`.env.EXAMPLE` shows a commented
  example) to avoid HF rate limits; without it "performance may vary."
- **Errors:** all errors flow through `AppError` enum (`src/errors.rs`)
  using miette/thiserror.
- **Snapshots:** insta and trycmd used for golden-master tests;
  review/regenerate via the just targets above.
- **Coverage:** CI sets `RUSTFLAGS="-Dwarnings -Cinstrument-coverage"` and
  uploads lcov to Codecov.

## CI / Workflow Conventions

- Commits must follow conventional commit format (commitizen pre-commit hook).
- Pre-commit runs: rustfmt, clippy, cargo-check,
  `cargo-deny check --all-features`, typos, gitleaks, yamlfmt.
- dprint formats JSON/MD/TOML; `_typos.toml` configures spell check.
- GitHub Actions are in `.github/workflows/*`
  (test, fmt, clippy, msrv 1.95, coverage, deps, security audit,
  pre-commit, typos).
