Project: Symposium Booklet CLI

Summary

Create a cross-platform Rust CLI `symposium-booklet` that:

- Reads either a single Excel workbook (abstracts + grouping sheets) or a directory containing two `.xlsx` workbooks.
- Produces Markdown files with YAML frontmatter organized by session.
- Generates localized Typst entry files and optionally invokes the local `typst` binary to render an A5 booklet PDF per locale.

Goals

- Validate input early (header detection, duplicate IDs, missing references) before writing files.
- Preserve abstract text as provided, with only light cleanup of section labels.
- Provide a minimal branded Typst layout with a table of contents and tag index.

Outputs

- One PDF per locale when Typst is available: `symposium-2026_<locale>.pdf`.

Constraints

- No translation of abstract content.
- Single-column layout on A5.
- No image handling in V1.
