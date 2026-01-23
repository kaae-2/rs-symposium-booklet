
07 Implementation Plan

Phases

-- Phase 1: Project skeleton, CLI, Excel parsing, validation, markdown & manifest generation, starter typst emitter (COMPLETED).
-- Phase 2: Typst invocation, localized typst generation, PDF rendering (COMPLETED; Typst output is sanitized and self-contained).
-- Phase 3: polish (font bundling, watch mode, serve previews) — TODO.

Modules (file-level) — status

- `src/main.rs` — CLI entrypoint and subcommand dispatch (implemented)
- `src/cli.rs` — clap definitions (implemented; Build/EmitTypst/Validate)
- `src/io/excel.rs` — reading and parsing Excel into domain structs (implemented: single-workbook or directory parsing, header detection, duplicate-id checks, locale detection)
- `src/model.rs` — data models: `Abstract`, `Session`, `Manifest` (implemented)
- `src/io/markdown.rs` — slugging and writing markdown files (implemented)
- `src/io/plan.rs` — dry-run planning model (implemented)
- `src/typst.rs` — typst emitter + optional invocation (implemented; minimal self-contained Typst with ToC + tag index)
- `src/validation.rs` — validation utilities and errors (implemented: reference checks)
- `src/log.rs` — tracing initialization (implemented)

Developer workflow

- Use `cargo run -- build --input data/sheets.xlsx --output out/` to test end-to-end; Typst render depends on local `typst` binary.
- `--dry-run` prints a human-readable plan and JSON plan to stdout.
- `--emit-parse-json` writes `output/tools_output/parse.json` summarizing parsed abstracts & sessions.
- Recommended next steps: run unit tests (add if missing), test build with finalized Excel files.

Milestones (status)

1) Skeleton + parsing + manifest generation — DONE
2) Markdown writer + manifest examples + dry-run plans — DONE
3) Typst emitter + localized labels + optional invocation — DONE
4) Polish: watch mode, serve previews — TODO

Notes

- Locale detection is implemented by checking `locale`/`sprog` columns and defaults to `da`.
- Unreferenced abstracts remain parsed but are not emitted unless referenced by a session.
