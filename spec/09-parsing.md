09 Parsing

Purpose

- Define how the tool parses the two input Excel files (abstracts and grouping/session) and the exact rules for validation and emitted parse JSON.

Inputs

- Two explicit Excel workbooks are required (the CLI and examples use explicit file paths):
  - Abstracts workbook (Sheet A, one row per abstract)
  - Grouping/session workbook (Sheet B, session definitions and poster-to-session mapping)

Required headers (Abstracts sheet)

- id — unique string identifier per abstract (required)
- title — text title (required)
- abstract / resumé — abstract body text (required)
- authors — author list (required)

Optional headers (Abstracts sheet)

- affiliation / hospital — free text
- Emne ord (3-5) or keywords — comma-separated index terms
- Take-home messages — optional short conclusions
- reference / doi — optional published reference
- literature / references — optional free-text references
- center — short centre code (optional)
- contact_email — author/contact email (optional)

Grouping sheet shapes

- Shape 1 (section rows + poster rows): The sheet contains session header rows (no ids) followed by poster rows where a cell contains one or more abstract ids (comma or semicolon separated). The parser treats rows with no id tokens as session headers.
- Shape 2 (tabular mapping): The sheet contains per-row mapping with columns such as `abstract_id`, `session_id`, `session_title`, `session_order`, `item_order`. When detected, this shape is authoritative for ordering.

Validation rules

- Missing required headers causes parser failure with a clear error listing the missing header and row context.
- Duplicate abstract ids in the abstracts workbook cause validation failure.
- All abstract ids referenced in the grouping workbook must exist in the abstracts workbook. If not, validation fails unless `--emit-parse-json` is used for diagnostic output (the CLI still reports missing references in the JSON).
- Unreferenced abstracts are placed into an `Unassigned` session (soft handling) by default.
- Discrepancies between `center`/`contact_email` fields in the two workbooks are captured in the parse JSON (report-only) and do not overwrite values.

Emit parse JSON

- The CLI supports `--emit-parse-json`. When provided the tool will:
  - Parse the inputs and validate references.
  - Write a JSON file at `OUTPUT_DIR/tools_output/parse.json` with the structure described below.
  - Exit without writing Markdown or running typst.

Parse JSON format

- Top-level fields:
  - summary: { num_abstracts_parsed: number, num_sessions: number }
  - abstracts: Array of Abstract objects (see data model below)
  - sessions: Array of Session objects (ordered)
  - discrepancies: optional array of { id, field, abstract_value, grouping_value }
  - missing_references: optional array of referenced ids not found in abstracts

- Abstract object (serialized):
  - id: String
  - title: String
  - authors: [String]
  - affiliation: Option<String>
  - center: Option<String>
  - contact_email: Option<String>
  - abstract_text: String
  - keywords: [String]
  - take_home: Option<String>
  - reference: Option<String>
  - literature: Option<String>
  - locale: String

- Session object:
  - id: String
  - title: String
  - order: number
  - items: [{ id: String, order: number }]

Notes

- Filenames and slugs are sanitized and truncated to avoid filesystem errors on Windows (long titles are truncated). The CLI ensures ASCII slug characters only.
- The parse JSON is intended as a diagnostic artifact and a machine-readable representation of the parsed model; downstream steps (markdown emission, typst generation) consume the same in-memory structures when run without `--emit-parse-json`.
