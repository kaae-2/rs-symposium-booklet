Project Implementation TODO
==========================

Purpose
-------
Concrete next work items to move the project from current prototype to spec-compliant Phase 1.

Remaining work (ordered by priority)

Priority 1 — Typst template enrichment (medium)
- Expand the typst emitter to support richer layout features (anchors/labels, internal links, and TOC macros).
- Decide on font bundling strategy and update template guidance (optional `templates/starter/fonts/`).
- Add tests that validate emitted `.typ` structure beyond the compile smoke test (e.g., TOC/index blocks, label escaping).

Priority 2 — Fixtures and parsing edge cases (medium)
- Add committed `.xlsx` fixtures (5–10 rows) or expand the generator to cover author parsing/affiliation derivation and long-title slug truncation.
- Add unit tests for author parsing (`;` and `og`) and affiliation extraction rules.

Priority 3 — CLI/UX polish (low)
- Improve exit codes for validation/build failures and use `--verbose` to increase logging detail.
- Consider additional subcommands (`render-typst`, `watch`) as follow-ups.

Suggested commit message (when batching)
- "feat: align parsing rules and typst output with spec"
