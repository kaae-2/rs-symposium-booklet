// Starter Typst template
// Minimal Typst template with semantic font fallbacks and placeholders.

set page(size: (148mm, 210mm), margin: 18mm)
set main-font: 'serif'
set heading-font: 'sans'

# {{TITLE}}

// Generated for locale: {{LOCALE}}

// Table of contents
{{TOC}}

// Main content (emitter replaces {{CONTENT}})
{{CONTENT}}

// Index appended by emitter
