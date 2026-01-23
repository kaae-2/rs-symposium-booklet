02 Inputs and Data Model

Sheets

- Abstracts sheet: header row detected within the first 12 rows. A row qualifies when it includes an `id` column and a `title`/`abstract`/`resum` column. Required column: `id`.
- Column detection is substring-based (case-insensitive). Title/authors/abstract columns fall back to adjacent columns if not explicitly matched.
- Optional columns: `keywords` / `nøgle` / `emne ord`, `take home` / `take-home`, `reference` / `doi`, `literature` / `references`, `center`, `email` / `contact`, `locale` / `sprog`.
- `locale` defaults to `da` when empty.
- Affiliation is derived from the authors field; there is no dedicated affiliation column.

Authors parsing

- Authors are split on `;` or `og`.
- Each author entry is split on commas; the first segment is treated as the author name and the last segment becomes an affiliation source. Unique affiliations are joined with `; `.

Grouping / sessions sheet

- Rows without abstract IDs are treated as session headers.
- Rows with one or more abstract IDs are session items. IDs can appear in any cell and can be comma/semicolon separated.
- Item order is derived from row order within the session.

Data model

- Abstract: id, title, authors, affiliation, center, contact_email, abstract_text, keywords, take_home, reference, literature, locale.
- Session: id, title, order, items (id + order).
- Manifest: event, sessions (minimal; no item map).

Validation rules

- Missing `id` column in the abstracts sheet aborts parsing.
- Duplicate abstract IDs abort.
- All referenced abstract IDs in sessions must exist; missing references abort.
