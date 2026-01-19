04 Typst Composition & Localization

Outputs

- `output/typst/book_en.typ` and `output/typst/book_da.typ` — generated typst entry files per locale.
- Localized strings stored in `templates/locales/en.toml` and `templates/locales/da.toml`.

Book layout

- Page size: A5
- Columns: single column for body text
- Typography: starter template will reflect Region H design guide — hospital palette/colors.
- Headings, session separators, table of contents, headers/footers with session name and page numbers.
- Clickable TOC and clickable internal links to abstracts if desired.

Index generation

- Build an alphabetical index from `keywords` found in manifest; typst template will place the index at the end.

Typst binary invocation

- If `typst` binary is available on PATH (or `--typst-bin` path provided), the tool runs:
  - `typst compile output/typst/book_en.typ -o output/symposium-<event>_en.pdf`
- If `typst` not available, tool writes typst files and logs a clear warning and the exact compile command to run.

Fonts & branding

- Template uses semantic font variables (serif for body, sans for headings). If specific Region H fonts are available locally they can be provided via `--fonts` or placed in `templates/fonts/` and referenced in the template. If not available, fall back to common system fonts.

Design guide

- The starter template will extract accessible color and spacing cues from the provided Region H design guide. Exact match of fonts/colors depends on availability and licensing.
