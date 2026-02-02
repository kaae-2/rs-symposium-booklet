// Starter Typst template
// Design system aligned with Region H tokens.

#set page(
  width: 148mm,
  height: 210mm,
  margin: (top: 20mm, bottom: 18mm, left: 18mm, right: 18mm),
)
#let brand-blue = rgb("#007dbb")
#let brand-navy = rgb("#002555")
#let brand-sky = rgb("#009ce8")
#let brand-muted = rgb("#e5f2f8")
#let brand-white = rgb("#ffffff")
#let page-footer = [
  #align(center)[
    #text(fill: rgb("#646c6f"), size: 8.5pt)[#context counter(page).display()]
  ]
]
#set text(font: "Mari", size: 10.5pt, fill: brand-white)
#set par(justify: true)
#set heading(numbering: none)
#show heading.where(level: 1): it => block(above: 0pt, below: 0pt)[
  #align(center)[
    #v(90pt)
    #text(size: 36pt, weight: "bold", font: "Mari", fill: white)[#it.body]
  ]
]
#show heading.where(level: 2): it => block(above: 10pt, below: 10pt)[
  #set text(size: 13pt, weight: "bold", font: "Mari")
  #text(fill: brand-blue)[#it.body]
]
#show heading.where(level: 3): it => block(above: 8pt, below: 4pt)[
  #set text(size: 11.5pt, weight: "semibold", font: "Mari")
  #text(fill: brand-navy)[#it.body]
]
#show strong: set text(weight: "semibold", fill: brand-navy)

// Front page
#set page(margin: 0mm, footer: none, fill: brand-navy)
#let page-w = 148mm
#let page-h = 210mm
#place(top + left)[
  #rect(width: page-w, height: page-h, fill: brand-navy, stroke: none)
]
#place(top + left, dx: 4.25mm, dy: 0mm)[
  #rect(width: 4.25mm, height: page-h, fill: brand-sky, stroke: none)
]
#place(top + left, dx: 0mm, dy: 31mm)[
  #box(width: page-w)[
    #align(center)[
      #set text(size: 34pt, weight: "bold")
      {{COVER_TITLE}}
    ]
  ]
]
#place(top + left, dx: 0mm, dy: 59mm)[
  #box(width: page-w)[
    #align(center)[
      #set text(size: 17pt, weight: "bold")
      {{COVER_SYMPOSIUM}}
    ]
  ]
]
#place(top + left, dx: 0mm, dy: 72mm)[
  #box(width: page-w)[
    #align(center)[
      #set text(size: 17pt, weight: "bold")
      {{COVER_DATE}}
    ]
  ]
]
#place(top + left, dx: 15.5mm, dy: 86mm)[
  #image("/templates/starter/images/cover.jpg", width: 116mm)
]
#place(top + left, dx: 0mm, dy: 155mm)[
  #box(width: page-w)[
    #align(center)[
      #set text(size: 13pt, weight: "bold")
      {{COVER_SUBTITLE}}
    ]
  ]
]
#place(top + left, dx: 15.5mm, dy: 190mm)[
  #image("/templates/starter/images/logo.png", height: 15.5mm)
]
#pagebreak()

// Body pages
#set page(
  margin: (top: 20mm, bottom: 18mm, left: 18mm, right: 18mm),
  fill: none,
  footer: page-footer,
)
#set text(font: "Mari", size: 10.5pt, fill: rgb("#333333"))

#show heading.where(level: 1): it => block(above: 0pt, below: 0pt)[
  #align(center)[
    #v(20pt)
    #text(size: 15pt, weight: "bold", font: "Mari", fill: brand-blue)[#it.body]
  ]
]
#set par(justify: false, spacing: 2pt)
#set text(size: 8.5pt)
#show outline.entry.where(level: 1): set text(weight: "bold")
#outline(title: [{{TOC_LABEL}}], depth: 2, indent: 1.1em)
#pagebreak()
#set text(size: 10.5pt)
#set par(justify: true)

{{CONTENT}}
