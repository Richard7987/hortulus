#import "/package/lib.typ": *
#set page(paper: "a4", margin: (x: 3.2cm, top: 8cm, bottom: 3cm))
#set text(lang: "es", size: 12pt)
#show: flora.with(seed: 12, layout: "title", colourway: "winter", gilt-dpi: 200)
#flora-title(align(center)[
  #text(26pt)[*Jardín de márgenes*] \
  #text(13pt, style: "italic")[Un ensayo sobre flora y papel]
])
#v(5cm)
#align(center)[Escrito con un paquete de Typst.]
