// Starter Typst template
// Design system aligned with Region H tokens.

#set page(
  width: 148mm,
  height: 210mm,
  margin: (top: 20mm, bottom: 18mm, left: 18mm, right: 18mm),
  footer: [
    #align(center)[
      #text(fill: rgb("#646c6f"), size: 8.5pt)[#context counter(page).display()]
    ]
  ],
)
#let brand-blue = rgb("#007dbb")
#let brand-navy = rgb("#002555")
#let brand-sky = rgb("#009ce8")
#let brand-muted = rgb("#e5f2f8")
#set text(font: "Libertinus Serif", size: 10.5pt, fill: rgb("#333333"))
#set heading(numbering: "1.")
#show heading.where(level: 1): it => block(above: 14pt, below: 8pt)[
  #let heading_base = text(size: 16pt, weight: "bold", font: "Source Sans 3", fill: brand-navy)[#it.body]
  #let available = page.width - page.margin.left - page.margin.right
  #let full = measure(heading_base).width
  #let scale_factor = if full > available { available / full } else { 1 }
  #box(width: available)[
    #place(left)[#scale(scale_factor, origin: left)[heading_base]]
  ]
  #line(length: 100%, stroke: (paint: brand-blue, thickness: 0.8pt))
]
#show heading.where(level: 2): it => block(above: 10pt, below: 4pt)[
  #set text(size: 13pt, weight: "bold", font: "Source Sans 3")
  #text(fill: brand-blue)[#it.body]
]
#show heading.where(level: 3): it => block(above: 8pt, below: 4pt)[
  #set text(size: 11.5pt, weight: "semibold", font: "Source Sans 3")
  #text(fill: brand-navy)[#it.body]
]
#show strong: set text(weight: "semibold", fill: brand-navy)

// Title block
#align(left)[
  #stack(
    spacing: 8pt,
    image("icons/logo.svg", width: 44pt),
    text(size: 20pt, weight: "bold", fill: brand-navy)[{{TITLE}}],
    text(size: 9pt, fill: brand-blue)[Locale: {{LOCALE}}]
  )
]

{{TOC}}

{{CONTENT}}
