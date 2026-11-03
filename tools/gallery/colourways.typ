#import "/package/lib.typ": *
#set page(paper: "a4", margin: (left: 2cm, right: 6cm, y: 2.5cm))
#show: flora.with(seed: 7, layout: "right", gilt-dpi: 200)
#for (i, c) in ("spring", "summer", "autumn", "winter", "ember", "plum", "rose", "original").enumerate() {
  if i > 0 { pagebreak() }
  flora-page(colourway: c, seed: 7 + i)
  heading[My document]
  parbreak()
  text(13pt, lorem(70))
}
