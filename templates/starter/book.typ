// Starter Typst template
// A minimal, safe Typst skeleton. The emitter will replace `{{CONTENT}}`
// with generated document fragments. Edit this file to add fonts, styles
// and layout rules.

// Page: A5 (Typst uses mm units)
set page(size: (148mm, 210mm), margin: 18mm)

// Fonts: prefer template-provided fonts; fall back to system fonts
// font family variables (semantic)
// body: serif, headings: sans
// Fonts
// Place font files (woff/ttf) in templates/starter/fonts/ and register
// them here using Typst's `import-font` or `font` functions. Example:
// import-font("templates/starter/fonts/SourceSerif4-Regular.ttf") as "SourceSerif";
// set main-font: "SourceSerif"; // fallback will be used if import fails

// Header & Footer
// The template provides simple header/footer placeholders. For a real
// Typst layout you can replace these with `page(header: ...)` and
// `page(footer: ...)` expressions or use a dedicated macro.
// Example (pseudocode):
// set page(header: (row(text[10pt]{SESSION NAME}, align:right, text[10pt]{Page {page} / {total}})), footer: (center(text[9pt]{© Hospital})))

// Document title
// (Rendered as a large heading at the start)
# {{TITLE}}

// Locale marker (informational)
// Generated for locale: {{LOCALE}}

// Table of contents (Typst `toc()` can be used in advanced templates)
// toc()

// Main content placeholder
{{CONTENT}}

// Index will be appended at the end by emitter
