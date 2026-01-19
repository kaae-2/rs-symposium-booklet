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
- Filenames: slugify title and prepend four-digit order within session (e.g., `0001-my-talk.md`). Ensure uniqueness by appending `-1`, `-2` if slugs collide.

Manifest

- JSON manifest with full session and item metadata and relative file paths. Example structure:
  - See spec/01-overview.md for a small example.

Index and keywords

- If `keywords` provided in abstracts, split by comma and include in manifest; typst template will build an index from these keywords.

Localization

- Each markdown file retains `locale: da` for content. Typst templates will read `manifest.json` and generate UI-localized labels per output locale. 