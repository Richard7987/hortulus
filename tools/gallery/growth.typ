#import "/package/lib.typ": *
#set page(paper: "a4", margin: (left: 2cm, right: 6cm, y: 2.5cm))
#show: flora.with(seed: 5, layout: "right", growth: sys.inputs.at("growth", default: "classic"), gilt-dpi: 200)
#for (i, colourway) in ("spring", "plum", "winter", "rose").enumerate() {
  if i > 0 { pagebreak() }
  flora-page(colourway: colourway)
  heading[My document]
  parbreak()
  text(11pt, lorem(470))
}
