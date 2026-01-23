06 Validation & Error Handling

Strict validation rules (fail-fast)

- The abstracts header row must be detected and must include an `id` column.
- Duplicate abstract IDs abort parsing.
- All abstract IDs referenced in sessions must exist; missing references abort.

Error output

- Errors include helpful context (e.g., duplicate id with row number, missing reference with session title).

Warnings

- When `typst` is not found, typst files are emitted and a compile command is logged.
