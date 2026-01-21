
07 Implementation Plan

Phases

-- Phase 1: Project skeleton, CLI, Excel parsing, validation, markdown & manifest generation, starter typst emitter (COMPLETED).
-- Phase 2: Typst invocation, localized typst generation, PDF rendering (PARTIAL: typst invocation is wired and emitter produces `.typ` files; however generated Typst contains syntax issues and needs sanitization before reliable rendering). Note: typst rendering depends on a local binary; the tool emits `.typ` files and prints the exact `typst compile` commands when the binary is absent.
-- Phase 3: polish (font bundling, watch mode, serve previews) — TODO.

Modules (file-level) — status

- `src/main.rs` — CLI entrypoint and subcommand dispatch (implemented)
- `src/cli.rs` — clap definitions (implemented)
- `src/io/excel.rs` — reading and parsing Excel into domain structs (implemented: single-workbook and two-workbook parsing, header detection, duplicate-id checks, locale detection)
- `src/model.rs` — data models: `Abstract`, `Session`, `Manifest` (implemented)
- `src/io/markdown.rs` — slugging and writing markdown files (implemented)
- `src/io/plan.rs` — dry-run planning model (implemented)
-- `src/typst.rs` — typst template generation and invocation (implemented: emits `.typ` files and plan entries; needs fixes to ensure output is valid Typst and templates are sanitized before merging)
- `src/validation.rs` — validation utilities and errors (implemented: validate_input + reference checks)
- `src/log.rs` — tracing initialization (implemented)
- Tests & fixtures (partial): unit and integration tests referenced in docs; add or expand where needed.

Key crates

- `clap` — CLI
- `calamine` — Excel (.xlsx) parsing
- `serde`, `serde_json`, `serde_yaml` — manifest and frontmatter
- `slug` — generate file-safe slugs
- `tracing` + `tracing-subscriber` — logging

Developer workflow

- Use `cargo run -- build --input data/sheets.xlsx --output out/` to test end-to-end; typst render depends on local `typst` binary.
- `--dry-run` produces a human-readable plan and JSON plan without writing files.
- `--emit-parse-json` writes `tools_output/parse.json` summarizing parsed abstracts & sessions.
- Recommended next steps: run unit tests (add if missing), test build with sample Excel files.

Milestones (status)

1) Skeleton + parsing + manifest generation — DONE
2) Markdown writer + manifest examples + dry-run plans — DONE
3) Typst emitter + localized templates + optional invocation — DONE (typst invocation is optional; rendering requires binary)
4) Polish: font bundling, watch mode, serve previews — TODO

Notes:
- Dry-run planning implemented: `src/io/plan.rs`, `src/io/markdown.rs::write_markdown_plan`, and `src/typst.rs::emit_typst_plan` produce both pretty and JSON plans used by `--dry-run`.
- Locale detection implemented in Excel parsing: `src/io/excel.rs` looks for `locale`/`sprog` headers and falls back to sensible defaults; abstracts carry a `locale` field.
- Duplicate-id detection and strict header checks are enforced by parsing (duplicate abstract ids return errors).
- Typst rendering: `src/typst.rs::maybe_run_typst` checks `typst --version` and runs `typst compile <file> -o <out.pdf>` per locale if available; otherwise it emits `.typ` files and logs the recommended commands.
- Unassigned abstracts: parsing groups unreferenced abstracts into an "Unassigned" session automatically.
