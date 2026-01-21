// Starter Typst template
// A minimal, safe Typst skeleton. The emitter will replace `{{CONTENT}}`
// with generated document fragments. Edit this file to add fonts, styles
// and layout rules.

// Page: A5 (Typst uses mm units)
set page(size: (148mm, 210mm), margin: 18mm)

// Fonts: prefer template-provided fonts; fall back to system fonts
// font family variables (semantic)
// body: serif, headings: sans

// Title
heading({{TITLE}})

// Locale marker
// Generated for locale: {{LOCALE}}

// Table of contents
toc()

{{CONTENT}}

// Index will be appended at the end by emitter
