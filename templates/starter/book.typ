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
// If you want to ship fonts with the template, place them in
// `templates/starter/fonts/` and uncomment the import-font lines below.
// Typst will fail if import-font points to a missing file, so keep
// these lines commented unless the font files are present.
// import-font("templates/starter/fonts/SourceSerif4-Regular.ttf") as "SourceSerif";
// import-font("templates/starter/fonts/SourceSans3-Regular.ttf") as "SourceSans";

// Semantic font settings (fall back to generic families)
set main-font: 'serif';
set heading-font: 'sans';

// Header & footer (simple implementation using Typst page settings)
// Replace session placeholder with a dynamic value if desired.
set page(
  header: (row(text[9pt]{Session: }, text[9pt bold]{SESSION NAME}, align: right, text[9pt]{Page {page} / {pages}})),
  footer: (center(text[9pt]{© Region H — Symposium 2026}))
)

// Document title
# (text[28pt bold]{ {{TITLE}} })

// Locale marker (informational)
// Generated for locale: {{LOCALE}}

// Table of contents — replaced by emitter when {{TOC}} present
{{TOC}}

// Main content placeholder
{{CONTENT}}

// Index will be appended at the end by emitter
