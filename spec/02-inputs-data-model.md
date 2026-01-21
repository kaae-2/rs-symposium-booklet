02 Inputs and Data Model

Sheets

- Sheet A (Abstracts): one row per abstract. Expected columns (tentative, to be finalized with samples):
  - `id` (string) — unique identifier per abstract
  - `title` (string)
  - `authors` (string) — author list; may be comma-separated
  - `affiliation` (string) — optional, may contain hospital
  - `abstract` (string) — plain text, Danish
  - `keywords` (string) — optional, comma-separated for index
- `locale` (string) — optional; default `da`
  - `locale` (string) — optional; default `da`. The parser will detect a `locale` (or Danish `sprog`) column when present and populate `Abstract.locale` accordingly.

- Sheet B (Inclusion & Sessions): drives which abstracts to include and ordering.
  Two supported shapes (tool will accept either; final shape to be decided when samples are provided):
  - Shape 1: one row per session, with a column `abstract_ids` containing a comma-separated list of abstract ids in the desired order, plus `session_id`, `session_title`, `session_order`.
  - Shape 2: one row per abstract mapping to a session: columns `abstract_id`, `session_id`, `session_title`, `session_order`, `item_order`.

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
- If both sheet shapes are present, prefer Shape 2 (per-abstract mapping).
