# tokenator — Project Context

Single-crate Rust CLI (edition 2024, MSRV 1.95) that counts LLM prompt tokens
using Hugging Face tokenizers.

## Architecture

- **Entrypoint:** `src/main.rs` — `#[tokio::main] async fn main()`.
  - Parses CLI via `src/cli.rs` (`Cli` struct: `model`, `file`, `prompt`,
    `--verbose`).
  - Calls `models::get_repo_id` to resolve the model name to an `<owner>/<name>`
    Hugging Face repo ID.
  - Calls `token::create_tokeniser` to download/cached-load `tokenizer.json`
    from Hugging Face.
  - Calls `prompt::get_prompt` to read prompt from file or CLI string.
  - Calls `token::count_tokens` to count tokens.
  - Writes formatted number to stdout via `num_format`.
- **Modules:** `cli`, `errors`, `models`, `prompt`, `token`, `utility`.
- **Errors:** all errors flow through `AppError` enum (`src/errors.rs`) using
  miette/thiserror. Error variants: `HfApi`, `InvalidRepoId`, `Io`, `Tokenizer`.
- **Models:** `data/models.json` maps names to `<owner>/<name>` Hugging Face
  repo ids. `src/models.rs:get_repo_id` enforces the `<owner>/<name>` pattern
  and suggests closest matches via `strsim::normalized_damerau_levenshtein`.
- **Tokenizers:** `tokenizers::Tokenizer` from `tokenizers` crate; Hugging Face
  file downloaded via `hf_hub::HFClient` and cached by hf-hub. Env
  `HUGGING_FACE_ACCESS_TOKEN` (`.env` shows a commented example) to avoid
  Hugging Face rate limits.
- **Logging:** `tracing` with `clap-verbosity-flag` verbosity levels (`-v` warn,
  `-vv` info, `-vvv` debug, `-vvvv` trace).
- **File reading:** `utility::read_file` limits to 10 MB; prompt text limited
  to ~20 MB in `prompt::get_prompt`.

## Developer Commands

| Command                                                          | Description                                                                                                              |
| ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `cargo build`                                                    | Build debug binary                                                                                                       |
| `./target/debug/tokenator <model> '<prompt>'` or `--file <path>` | Run the CLI                                                                                                              |
| `cargo test --lib`                                               | Unit tests only (no network)                                                                                             |
| `cargo test`                                                     | Unit + integration (integration tests spawn the binary and download HF tokenizers — network required, may be slow/flaky) |
| `cargo clippy -- -D warnings`                                    | CI-enforced lints                                                                                                        |
| `cargo fmt --check`                                              | Edition 2024, Unix newlines                                                                                              |
| `just integration-tests`                                         | `cargo test --test '*'`                                                                                                  |
| `just trycmd-snapshot-overwrite`                                 | Regenerate trycmd snapshots (`tests/cmd/*.toml`)                                                                         |
| `just trycmd-snapshot-dump`                                      | Dump trycmd snapshots to `dump/` (gitignored)                                                                            |
| `just insta-snapshot-clean`                                      | Delete unreferenced insta snapshots                                                                                      |
| `just insta-snapshot-review`                                     | Review insta snapshots (`cargo insta review`)                                                                            |
| `just coverage`                                                  | grcov coverage (needs llvm from brew)                                                                                    |

## Testing

- **Unit tests:** in-module `#[cfg(test)]` blocks using `insta` for JSON
  snapshots (`src/snapshots/`).
- **Integration tests:** `tests/integration/ui.rs` spawn
  `./target/debug/tokenator` via `rexpect` and download HF tokenizer files.
  Requires network.
- **Trycmd tests:** `tests/cmd/help.toml` (and `help.stdout`, `help.stderr`) for
  golden-master CLI output.
- Test style: arrange/act/assert comments in tests (no `act!`/`assert!` macros).

## CI / Workflow Conventions

- **Commits:** conventional commit format (commitizen pre-commit hook).
- **Pre-commit runs:** rustfmt, clippy, cargo-check,
  `cargo-deny check --all-features`, typos, gitleaks, yamlfmt.
- **dprint:** formats JSON/MD/TOML.
- **Spell/ignore lists:** `_typos.toml` and `yek.yaml`.
- **GitHub Actions:** workflows in `.github/workflows/*` (test, fmt, clippy,
  msrv 1.95, coverage, deps, security audit, pre-commit, typos).
- **Coverage:** CI sets `RUSTFLAGS="-Dwarnings -Cinstrument-coverage"` and
  uploads lcov to Codecov.

## How to Build and Run

```bash
cargo build
./target/debug/tokenator qwen3:1.7b 'Why is the sky blue?'
# or
./target/debug/tokenator --file prompt.txt
```

Set `HUGGING_FACE_ACCESS_TOKEN` to avoid HF rate limits.

## Models

Edit `data/models.json` to add/remove models. Each entry:
`{ "name": "<alias>", "hf": "<owner>/<name>" }`. The `hf` value must match the
`<owner>/<name>` pattern.

## Snapshots

- insta snapshots in `src/snapshots/` — regenerate via
  `just insta-snapshot-review` or set `INSTA_UPDATE=always`.
- trycmd snapshots in `tests/cmd/*.toml` plus `.stdout`/`.stderr`
  files — regenerate via `just trycmd-snapshot-overwrite`.

## Key Conventions

- No comments added unless asked (per project style).
- Code style: edition 2024, Unix newlines, rustfmt.
- Error handling: miette diagnostics with `#[help]` advice and
  `#[error("{detail}")]`.
- Tests use `assert_fs::TempDir` for temp files with `unwrap()` on close.
- `clippy` pedantic warnings enabled but specific lints denied/allowed in
  `Cargo.toml` `[lints.clippy]`.
