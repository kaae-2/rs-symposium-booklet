03 Output Layout

Filesystem layout

- `output/manifest.json` — master manifest describing sessions and items
- `output/typst/` — generated typst files and templates
- `output/<session-slug>/NNNN-<slug>.md` — markdown files per abstract
- `output/symposium-<event>_en.pdf` and `_da.pdf` — generated PDF booklets

Markdown file convention

- YAML frontmatter required with fields:
  - `id`, `title`, `authors` (array), `session`, `order`, `locale`
- Body: the abstract text (plain text), with minimal normalization to paragraphs.
- Author parsing: authors are split on `;` or `og`. Each author entry is split on commas; the first segment is treated as the author name and the last segment is aggregated into the `affiliation` field.
- Filenames: slugify title and prepend four-digit order within session (e.g., `0001-my-talk.md`). Ensure uniqueness by appending `-1`, `-2` if slugs collide. Slugs are ASCII-only and truncated to avoid Windows path length issues (session slug ~60 chars, title slug ~80 chars).

Manifest

- JSON manifest with full session and item metadata and relative file paths. Example structure:
  - See spec/01-overview.md for a small example.

Index and keywords

- If `keywords` provided in abstracts, split by comma and include in manifest; typst template will build an index from these keywords.

Localization

- Each markdown file retains `locale: da` for content. Typst templates will read `manifest.json` and generate UI-localized labels per output locale. 

Notes on current implementation

- The emitter writes `output/typst/book_<locale>.typ` as a minimal, validated Typst document (no template merge). A compile smoke test is included to ensure Typst output stays parseable.
