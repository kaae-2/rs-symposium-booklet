04 Typst Composition & Localization

Outputs

- `output/typst/book_<locale>.typ` — generated Typst entry files per locale.
- Localized strings stored in `templates/starter/locales/<locale>.toml` (defaults in code if missing).

Book layout

- Page size: A5
- Columns: single column for body text
- Minimal branded layout with gradient cover, section separators, and page numbers.
- Table of contents uses a Danish heading (`Indholdsfortegnelse`) followed by an outline title from `toc_label`.
- The tag index section is a level-1 heading (from `tag_index_label`) so it appears in the ToC.
- Abstract titles are link targets; the tag index links to abstracts with page numbers.

Typst binary invocation

- When Typst is available (or `--typst-bin` provided), the tool runs:
  - `typst compile --font-path templates/starter/fonts/TTF output/typst/book_<locale>.typ output/symposium-2026_<locale>.pdf`
- When Typst is missing, the tool writes `.typ` files and logs the exact compile command.

Fonts & branding

- Body font: Libertinus Serif.
- Heading font: Source Sans 3 (bundled in `templates/starter/fonts/TTF`).
