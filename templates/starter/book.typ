// Starter Typst template
// Minimal Typst template with semantic font fallbacks and placeholders.

set page(size: (148mm, 210mm), margin: 18mm)
set main-font: 'serif'
set heading-font: 'sans'

// Header: left logo (from extracted assets) and centered title
row(
  embed("spec/design-guide/icons/image_1_1.jpg", width: 60pt),
  center(text[18pt bold]{ {{TITLE}} })
)

// Generated for locale: {{LOCALE}}

// Table of contents
// Small TOC icon
embed("templates/starter/icons/toc.svg")
{{TOC}}

// Main content (emitter replaces {{CONTENT}})
{{CONTENT}}

// Index appended by emitter (icon)
embed("templates/starter/icons/index.svg")
