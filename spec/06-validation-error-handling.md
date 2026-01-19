06 Validation & Error Handling

Strict validation rules (fail-fast)

- Required headers in Sheet A (`id`, `title`, `authors`, `abstract`) must exist.
- Required headers in Sheet B depend on shape chosen; if using Shape 2 (per-abstract mapping) required headers: `abstract_id`, `session_id`, `session_title`, `item_order`.
- Duplicated abstract IDs in Sheet A cause abort with details.
- References from Sheet B to abstract IDs must resolve; missing references cause abort with sheet and row info.
- Invalid cell types (e.g., non-numeric `item_order`) cause abort.

Error output

- Errors print with contextual detail: sheet name, row number, column name, offending value and suggestion.
- When `--dry-run` validation fails, no files are written and exit code is non-zero.

Warnings

- Missing optional columns (e.g., `keywords`, `affiliation`) produce warnings but do not abort.
- When `typst` binary not found, tool emits typst files and logs a clear warning and the exact `typst compile` command to run.

Recovery & diagnostics

- Suggest a `--report` flag (future) to write a JSON diagnostics file with full failure stack for triage.
