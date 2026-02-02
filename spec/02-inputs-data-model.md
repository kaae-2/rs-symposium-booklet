02 Inputs and Data Model

Sheets

- Abstracts sheet: header row detected within the first 12 rows. A row qualifies when it includes an `id` column and a `title`/`abstract`/`resum` column. Required columns: `id`, `tema`, `type`, `order`.
- Column detection is substring-based (case-insensitive) for existing fields, but `tema`, `type`, and `order` are matched by exact column name (case-insensitive).
- Optional columns: `keywords` / `nøgle` / `emne ord`, `take home` / `take-home`, `reference` / `doi` (incl. "Reference hvis studiet er publiceret. (link eller DOI)"), `literature` / `references`, `center`, `email` / `contact`, `locale` / `sprog`.
- `locale` defaults to `da` when empty.
- Affiliation is derived from the presenters field; there is no dedicated affiliation column.

Presenters parsing

- Presenters are split on `;` or `og`.
- Each presenter entry is split on commas; the first segment is treated as the presenter name and the last segment becomes an affiliation source. Unique affiliations are joined with `; `.

Ordering fields

- `tema` must be one of: `Miljø`, `Teknologi`, `Organisation`.
- `type` must be one of: `Poster`, `Mundtlig`.
- `order` must be an integer.

Data model

- Abstract: id, title, tema, type, order, presenters, affiliation, center, contact_email, abstract_text, abstract_sections, keywords, take_home, reference, literature, locale.
- Session: id, title, tema, type, order, items (id + order), derived by grouping abstracts on tema + type.
- Manifest: event, sessions (minimal; no item map).

Validation rules

- Missing `id` column in the abstracts sheet aborts parsing.
- Missing `tema`, `type`, or `order` columns aborts parsing.
- Invalid `tema`, `type`, or `order` values abort parsing.
- Duplicate abstract IDs abort.
- Rows with all of `tema`, `type`, and `order` missing are skipped with a warning.
