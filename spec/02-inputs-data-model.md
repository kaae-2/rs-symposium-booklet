02 Inputs and Data Model

Sheets

- Sheet A (Abstracts): one row per abstract. The parser detects headers by substring match (case-insensitive) and expects:
  - Required: `id`, `title` / `titel`, `authors` / `forfatter`, `abstract` / `resumé`
  - Optional: `affiliation` / `hospital` / `afdeling`, `keywords` / `emne ord`, `take home` / `take-home`, `reference` / `doi`, `literature` / `references`, `center`, `email` / `contact`, `locale` / `sprog`
  - Any other columns in the workbook are ignored.

- Authors parsing: the authors field may contain multiple authors separated by `;` or `og`. Each author entry is split on commas; the first segment is treated as the author name and the last segment is treated as the affiliation source. The tool aggregates these affiliation segments into the `affiliation` field.

- `locale` defaults to `da` when no locale/sprog column is present.

- Sheet B (Inclusion & Sessions): drives which abstracts to include and ordering.
  - The grouping workbook uses session header rows (no ids) followed by item rows that include one or more abstract ids.
  - The parser uses the first matching grouping sheet (e.g., `gruppering på poster`) and ignores additional grouping sheets in the workbook.

Data Model

- Abstract struct:
  - id: String
  - title: String
  - authors: Vec<String>
  - affiliation: Option<String>
  - abstract_text: String
  - keywords: Vec<String>
  - locale: String

- Session struct:
  - id: String
  - title: String
  - order: u32
  - items: Vec<AbstractRef> (ordered)

- Manifest
  - event: String
  - sessions: Vec<Session>
  - items: Map<id, Abstract>

Validation rules

- Required headers must exist; missing headers cause strict validation failure.
- All abstract ids referenced in Sheet B must exist in Sheet A; otherwise abort.
- Duplicate ids in Sheet A abort.
- Grouping sheet order is derived from row order and detected abstract id tokens.
