03 Output Layout

Filesystem layout

- `output/manifest.json` — minimal manifest describing sessions.
- `output/typst/book_<locale>.typ` — generated Typst entry files per locale.
- `output/<session-slug>/NNNN-<slug>.md` — markdown files per abstract.
- `output/symposium-2026_<locale>.pdf` — generated PDF booklets when Typst is available.

Build behavior

- `build` wipes the entire output directory before emitting new files; `--dry-run` includes the delete action in the plan.

Markdown file convention

- YAML frontmatter fields:
  - `id`, `title`, `authors` (array), `affiliation` (optional), `session`, `order`, `locale`
  - Optional: `keywords` (array), `take_home`, `sections` (array of `{label,text}`)
- Body: abstract text joined from section bodies (labels removed).
- Filenames: slugify title and prepend four-digit order within session (e.g., `0001-my-talk.md`). Ensure uniqueness by appending `-1`, `-2` if slugs collide. Slugs are ASCII-only and truncated to avoid Windows path length issues (session slug ~60 chars, title slug ~80 chars).

Manifest

- JSON manifest with:
  - `event`: `symposium-2026`
  - `sessions`: array of { id, title, slug, order, count }

Index and keywords

- If `keywords` are provided in abstracts, Typst builds a tag index at the end of the booklet.
- The tag index is emitted as a level-1 heading so it appears in the table of contents.

Localization

- UI labels are loaded from `templates/starter/locales/<locale>.toml`, with defaults in code.

Notes on current implementation

- Typst output is a self-contained, minimal document with embedded styles.
- Mari is bundled in `templates/starter/fonts/TTF` and is used for body and heading typography via `--font-path`.
- The ToC is preceded by a Danish heading (`Indholdsfortegnelse`) and nudged upward on the page.
