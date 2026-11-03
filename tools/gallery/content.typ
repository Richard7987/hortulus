#import "/package/lib.typ": *
#set text(size: 12pt)
#set page(paper: "a4", margin: (left: 2cm, right: 6cm, y: 2.5cm))
#show: flora.with(seed: 33, layout: "right", colourway: "spring", gilt-dpi: 200)

= My document
#lorem(60)
#v(8cm)
#lorem(60)

#pagebreak()
#flora-page(colourway: "autumn")
= My document
#lorem(35)
#figure(
  table(
    columns: 3,
    table.header[*Ink*][*Group*][*Part*],
    [kon-peki], [blue], [hero],
    [momiji], [warm], [blooms],
    [shin-ryoku], [green], [leaves],
  ),
  caption: [A table.],
)
$ integral_0^1 x^2 dif x = 1 / 3 $
#lorem(25)

#pagebreak()
#flora-page(colourway: "plum")
= My document
#lorem(40)
#figure(
  placement: bottom,
  rect(width: 100%, height: 6cm, fill: luma(232)),
  caption: [A floating figure.],
)
#lorem(30)

#set page(margin: (inside: 2cm, outside: 6cm, y: 2.5cm), columns: 2)
#flora-page(layout: "split", colourway: "rose")
#lorem(280)
