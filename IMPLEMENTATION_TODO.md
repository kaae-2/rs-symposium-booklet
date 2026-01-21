Project Implementation TODO
==========================


Purpose
-------
Concrete next work items to move the project from current prototype to spec-compliant Phase 1.


Remaining work (ordered by priority)

Priority 1 — Dry-run semantics and planning (high)
- Implement `--dry-run` behavior that performs validation and emits a human-readable plan of filesystem actions without writing files. (DONE)
- Produce a plan that lists directories to create, markdown files (path + frontmatter summary), typst files to be emitted, and the final manifest updates. (DONE)
- Files changed: `src/io/mod.rs` (drive dry-run), `src/io/markdown.rs` (separate plan generation vs write), `src/typst.rs` (emit plan form), `src/io/plan.rs` (new).
- Estimate: 1–2 hours.

Priority 2 — Read & preserve `locale` from input (medium)
- Detect `locale` header in Excel and populate `Abstract.locale` (DONE).
- Ensure `locale` from the model is preserved in frontmatter (already done by markdown writer).
- Added tests: `tests/dry_run_and_locale.rs` covers locale detection and plan generation.
- Files changed: `src/io/excel.rs` (locale detection), `src/io/markdown.rs` (frontmatter already uses `abs.locale`), `tests/dry_run_and_locale.rs` (new).
- Estimate: 30–60 minutes.

Priority 3 — Typst templates & manifest-driven generation (medium)
 - Add starter templates under `templates/starter/` and update `src/typst.rs` to generate typst by reading `output/manifest.json` and per-abstract frontmatter. (IMPLEMENTED — emitter now builds `.typ` per-locale and merges content into a minimal, validated Typst document)
- Support UI localization (en/da) for labels in the template. (IMPLEMENTED — `templates/starter/locales/{en,da}.toml` present)
 - Keep PDF rendering optional; when not running, print exact `typst compile` commands the user can run. (IMPLEMENTED — `maybe_run_typst` logs commands and runs local `typst` when available)
 - Remaining work: expand the typst emitter to support richer template merging (anchors, internal links, TOC macros), add unit tests that assert emitted `.typ` files parse/compile, and provide clear font bundling or import guidance for reliable PDF outputs.
- Files to add/change: `templates/starter/`, `src/typst.rs`.
- Estimate (remaining): 1–3 hours.

Priority 4 — Tests and fixtures (remaining)
- Add small fixture workbooks (`data/fixtures/`) with 5–10 rows that exercise: duplicate IDs, missing refs, locale column, duplicate titles to trigger slug suffixing, and session grouping. (PARTIAL: test scaffold and fixture generator exist; no committed .xlsx fixtures yet)
- Add unit tests for `parse_abstracts_from_rows`, header detection, and dry-run plan output. Also add tests for `emit_typst` output (validate generated `.typ` structure) and a smoke test that runs `typst compile` when binary is present.
- Files to add: `tests/`, `data/fixtures/`.
- Estimate: 2–4 hours.

Priority 5 — CLI and UX polish (low)
- Improve exit codes for validation/build failures and use `--verbose` to increase logging detail.
- Consider additional subcommands (`render-typst`, `watch`) as follow-ups.
- Files: `src/cli.rs`, `src/log.rs`.
- Estimate: 1–2 hours.

Deliverables for Phase 1 (updated)
- Validation and `validate` command present and working.
- `build` with a proper `--dry-run` plan output (implemented).
- Markdown files written following slug/collision rules and `manifest.json` (implemented).
 - Starter typst templates + manifest-driven `.typ` files emitted (implemented). The emitter currently writes a minimal, validated `.typ` per locale which avoids copying template comments into generated output; richer template features (anchors, fancy TOC, bundled fonts) remain to be implemented and tested.

Additional notes / immediate next work
 - Expand typst emitter features: optional anchors/labels, internal links, robust TOC generation using Typst macros, and tests asserting emitted `.typ` parseability and (optionally) successful `typst compile` in CI when the binary is available.
 - Add font files under `templates/starter/fonts/` (open-source defaults or Region H licensed fonts) and enable `import-font(...)` lines in the template.
 - Add committed `.xlsx` fixtures and unit tests for `emit_typst` and end-to-end smoke tests that run `typst compile` when available.

Suggested small milestones (apply sequentially)
1) Implement dry-run planning output and wire `--dry-run` to print the plan (no writes).
2) Read `locale` from input and ensure it flows through to frontmatter and typst generation.
3) Add starter typst templates and improve `emit_typst` to use the manifest and templates.
4) Add fixtures and tests covering parsing, validation failures, slug collisions, and dry-run.

Suggested commit message
- "feat: add dry-run planning, detect locale column, improve typst plumbing"

Files likely to change
- Update: `src/io/mod.rs`, `src/io/markdown.rs`, `src/io/excel.rs`, `src/typst.rs`, `src/validation.rs` (minor), `src/cli.rs` (minor)
- Add: `templates/starter/`, `data/fixtures/`, `tests/`
