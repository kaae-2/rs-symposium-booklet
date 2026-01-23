Project Implementation TODO
==========================

Purpose
-------
Concrete next work items to move the project from the current implementation to the final polish phase.

Remaining work (ordered by priority)

Priority 1 — Tests and fixtures (medium)
- Add committed `.xlsx` fixtures (5–10 rows) or expand the generator to cover author parsing/affiliation derivation and long-title slug truncation.
- Add unit tests for author parsing (`;` and `og`) and affiliation extraction rules.
- Add tests that validate emitted `.typ` structure (TOC/index blocks, label escaping, tag index linking).

Priority 2 — UX polish (low)
- Improve exit codes for validation/build failures and use `--verbose` to increase logging detail.
- Consider additional subcommands (`watch`) as follow-ups.

Suggested commit message (when batching)
- "chore: refresh docs and todo list"
