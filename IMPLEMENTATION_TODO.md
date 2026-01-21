Project Implementation TODO
==========================

Purpose
-------
Concrete next work items to move the project from current prototype to spec-compliant Phase 1.

Priority 1 — Strict validation (done)
- Header detection is now strict and fails fast for required headers (`id`, `title`, `abstract`).
- Duplicate abstract IDs abort parsing (warnings converted to errors).
- `validation::validate_input` implemented and wired to the `symposium-booklet validate` command.
- Files changed: `src/io/excel.rs`, `src/validation.rs`, `src/io/mod.rs`, `src/main.rs`.

Priority 2 — Dry-run semantics and planning (medium)
- Implement `--dry-run` to validate and print planned filesystem writes without writing files.
- Produce a human-readable "plan" (list of files and actions).
- Files: `src/io/mod.rs`, `src/io/markdown.rs`, `src/typst.rs`.
- Estimate: 1–2 hours.

Priority 3 — Filename uniqueness / slug collisions (medium)
- Ensure unique markdown filenames per session by detecting collisions and appending `-1`, `-2` etc.
- Files: `src/io/markdown.rs`.
- Estimate: 1 hour.

Priority 4 — Typst templates & localization (medium)
- Add starter templates in `templates/starter/`.
- Implement manifest-driven typst generation that supports UI localization (en/da).
- Ensure `locale` column is read if present and preserved in frontmatter.
- Files: `src/typst.rs`, new `templates/` directory.
- Estimate: 2–4 hours.

Priority 5 — Tests and fixtures (some done)
- Added unit tests for header detection and duplicate-id handling under `tests/validation_tests.rs`.
- Still to add: fixture Excel workbooks under `data/fixtures/` and tests for slug collision behavior.
- Files changed: `tests/validation_tests.rs`, `src/io/excel.rs` (test helpers).

Priority 6 — CLI and UX polish (low)
- Improve exit codes on validation failures.
- Revise logging; add `--verbose` behaviour if needed.
- Consider `render-typst` and `watch` subcommands as follow-ups.
- Files: `src/cli.rs`, `src/log.rs`.
- Estimate: 1–2 hours.

Deliverables for Phase 1
- Strict validation and `validate` command working.
- `build` with `--dry-run` plan output.
- Markdown files written following slug/collision rules and `manifest.json`.
- Starter typst files emitted; PDF rendering still optional (requires local `typst`).

Suggested small milestones (apply sequentially)
1) Implement strict validation and `validate` command.
2) Fix dry-run semantics and implement planning output.
3) Add slug collision handling and adjust markdown writer.
4) Add starter typst templates and improve `emit_typst`.
5) Add unit tests and fixtures.

Suggested commit message
- "feat: enforce strict validation, add implementation TODO, update spec README"

Notes
- I will not change existing files that the repo owner has modified elsewhere; edits are limited to spec files and code paths described above.
- After you approve I can apply the patch (create the TODO file, delete `spec/08-todo-next-steps.md`, update `spec/README.md`) and run `cargo test` / `cargo run` as requested.

Files to be created/changed
- Add: `spec/08-implementation-todo.md`
- Delete: `spec/08-todo-next-steps.md` (already removed)
- Update: `spec/README.md` (replace reference to `IMPLEMENTATION_TODO.md`)
