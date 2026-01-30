08 Parsing

Purpose

- Define how the tool parses input Excel files and the rules used for validation and parse JSON output.

Inputs

- A single workbook containing both abstracts and sessions sheets, OR a directory containing two `.xlsx` files (abstracts + sessions).
- When given a directory, the parser prefers filenames containing `with_ids`/`afsluttede` for abstracts and `kopi`/`grupper`/`final` for sessions, otherwise it uses the first two `.xlsx` files.

Abstracts sheet

- The sheet is selected by name heuristics (`afsluttede`, `abstract`, `resum`).
- Header row detection scans the first 12 rows for an `id` column and a `title`/`abstract`/`resum` column.
- Required column: `id`.
- Column detection (case-insensitive substrings):
  - title/titel, authors/author/forfatter, abstract/resum
  - keywords/nøgle/emne ord
  - take home/take-home
  - reference/published/doi
  - literature/litterature/references
  - center/centre
  - email/kontakt/contact
  - locale/sprog
- If a column is not found, the parser falls back to adjacent columns.
- Abstract text is split into sections based on common labels (Background, Objective, Methods, etc.).
- Section labels are normalized by trimming whitespace and stripping trailing commas, periods, and colons.
- If the abstract starts without a known label, a default label is inserted (locale-based: `Resumé` for `da`, `Abstract` otherwise).
- Locale defaults to `da`.

Authors parsing

- Authors are split on `;` or `og`.
- Each author entry is split by comma; the first segment is the author name and the last segment becomes a unique affiliation entry.

Grouping/session sheet

- The sheet is selected by name heuristics (`gruppering`, `poster`, `session`, `include`).
- Rows without abstract IDs are treated as session headers.
- Rows containing known abstract IDs are treated as items; IDs can appear in any cell and may be comma/semicolon separated.
- Item order is based on row order within the session.

Validation rules

- Missing `id` column aborts.
- Duplicate abstract IDs abort.
- Missing references in sessions abort.

Emit parse JSON

- `build --emit-parse-json` writes `output/tools_output/parse.json` and exits without writing Markdown/Typst.
- The tool still validates references before writing parse JSON.

Parse JSON format

- Top-level fields:
  - summary: { num_abstracts_parsed: number, num_sessions: number }
  - abstracts: Array of Abstract objects
  - sessions: Array of Session objects (ordered)

Abstract object (serialized)

- id, title, authors, affiliation, center, contact_email, abstract_text, abstract_sections, keywords, take_home, reference, literature, locale

Session object

- id, title, order, items: [{ id, order }]
