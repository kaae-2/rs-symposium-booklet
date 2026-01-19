Project: Symposium Booklet CLI

Summary

Create a cross-platform Rust CLI `symposium-booklet` that:

- Reads two Excel sheets (abstracts + inclusion/session mapping).
- Produces a structured set of Markdown files with YAML frontmatter organized by session.
- Generates localized Typst files (English/Danish UI) and invokes the local `typst` binary to render a single A5 booklet PDF per locale.

Goals

- Strict validation: abort on missing required data or invalid references before writing files.
- Maintain abstract text as-is (Danish); only UI strings and Typst templates are localized.
- Starter Typst template follows the Region H design guide for hospital branding.
- Output: one PDF per locale: `symposium-<event>_en.pdf` and `symposium-<event>_da.pdf`.

Constraints

- No translation of abstract content.
- Single-column layout on A5.
- No image handling in V1.
