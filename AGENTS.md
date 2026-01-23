AGENTS

Purpose

This document records the "agents" (roles and automated behaviours) that will be used while building the Symposium Booklet CLI. It captures how the assistant (OpenCode) will act, what it will ask, what it will produce, and how it will interact with you and the repository during development.

Primary agent: OpenCode (assistant)

Responsibilities

- Interview & gather requirements: ask concise, targeted questions about localization, ordering, and branding (Excel schema is finalized).
- Produce artefacts: spec files, example manifests, minimal Typst emitters, scaffold Rust project files, unit tests and example outputs.
- Implement features when asked: parse Excel sheets, validate data strictly, write Markdown files, emit self-contained Typst entry files, and call the local `typst` binary to render PDFs.
- Log and report: provide clear validation errors, runtime warnings (e.g., when `typst` is missing), and a manifest describing exported content.

Behaviour & interaction model

- The Excel schema is finalized; update specs if the input sheets change.
- Use strict validation by default: missing `id` column, duplicate IDs, or unresolved references abort before filesystem writes.
- If `--dry-run` is used, validate and print planned actions without writing files.
- When `typst` is not found on PATH, the agent will still emit typst files and print the exact `typst compile` command to run; it will not attempt to download or install binaries.
- Preserve abstract text exactly as provided; do not attempt translation or modification of Danish content (only UI text/localized labels are generated separately).

Tools and libraries (implementation plan)

- Rust crates: `clap`, `calamine` (xlsx parsing), `serde`/`serde_json`/`serde_yaml`, `slug`, `tracing` / `tracing-subscriber`.
- Typst: the agent will generate self-contained `.typ` files and will call the local `typst` binary via `std::process::Command` if present.

Files the agent will create (examples)

- `spec/` (contains the split spec files and this `AGENTS.md`).
- `templates/starter/` (locales, optional fonts).
- `output/` (manifest, per-session markdown files, generated typst files, and final PDF when typst is run).
- `src/` Rust modules for CLI, Excel parsing, model, markdown emission, typst generation and validation.

Safety & repo hygiene

- The agent will never run destructive Git commands (no resets, no force push) and will not modify unrelated files the user has changed.
- The agent will not commit changes unless explicitly requested.
- The agent avoids adding secrets to the repo.

Communication

- The agent will summarise each significant change (files added or modified) and present exact paths for easy inspection.
- For ambiguous decisions, the agent will provide a short set of options and ask the user to choose.

Next steps the agent will request from you

1. Confirm whether the bundled Source Sans 3 + Libertinus Serif pairing is acceptable.
2. Provide any branding constraints or additional layout requirements.
