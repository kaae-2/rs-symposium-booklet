07 Implementation Plan

Phases

- Phase 1: Project skeleton, CLI, Excel parsing, validation, markdown & manifest generation, starter typst emitter (no rendering).
- Phase 2: Typst invocation, localized typst generation, PDF rendering, index generation.
- Phase 3: polish (font bundling, watch mode, serve previews).

Modules (file-level)

- `src/main.rs` — CLI entrypoint and subcommand dispatch
- `src/cli.rs` — clap definitions
- `src/io/excel.rs` — reading and parsing Excel into domain structs
- `src/model.rs` — data models: Abstract, Session, Manifest
- `src/io/markdown.rs` — slugging and writing markdown files
 - `src/io/plan.rs` — dry-run planning model
- `src/io/typst.rs` — typst template generation and invocation
- `src/validation.rs` — validation utilities and errors
- `src/log.rs` — tracing initialization

Key crates

- `clap` — CLI
- `calamine` — Excel (.xlsx) parsing
- `serde`, `serde_json`, `serde_yaml` — manifest and frontmatter
- `slug` — generate file-safe slugs
- `tracing` + `tracing-subscriber` — logging

Developer workflow

- `cargo run -- build --input data/sheets.xlsx --output out/` to test end-to-end
- Unit tests for `excel.rs` parsing and `markdown.rs` slugging

Milestones

1) Skeleton + parsing + manifest generation
2) Markdown writer + manifest examples
3) Typst emitter + localized templates

Notes:
- Dry-run planning implemented: see `src/io/plan.rs`, `src/io/markdown.rs::write_markdown_plan`, and `src/typst.rs::emit_typst_plan` which produce a human-readable and JSON plan when `--dry-run` is used.
 - Dry-run planning implemented: see `src/io/plan.rs`, `src/io/markdown.rs::write_markdown_plan`, and `src/typst.rs::emit_typst_plan` which produce a human-readable and JSON plan when `--dry-run` is used.
 - Locale detection implemented: `src/io/excel.rs` detects an optional `locale` or `sprog` header and populates `Abstract.locale`. Tests added in `tests/dry_run_and_locale.rs`.
4) Typst invocation + PDF building

Estimated times (rough)

- Phase 1: 6–10 hours
- Phase 2: 4–8 hours
- Phase 3: 3–6 hours
