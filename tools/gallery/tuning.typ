#import "/package/lib.typ": *
#set page(paper: "a4", margin: (left: 2cm, right: 6cm, y: 2.5cm))
#show: flora.with(seed: 21, layout: "right", colourway: "summer", gilt-dpi: 200)
#for (i, opts) in (
  (density: 0.4),
  (density: 2),
  (visitors: 4),
  (mix: (lily: 3, bloom: 0, umbel: 0)),
  (palette: (hero: "kon-peki", blooms: "kon-peki", leaves: "kon-peki", leaflets: "kon-peki", sprays: "kon-peki")),
  (palette: (hero: "momiji", blooms: "momiji", leaves: "shin-ryoku", leaflets: "shin-ryoku", sprays: "shin-ryoku")),
  (ornaments: (moon: 5, night: 0, sun: 0, planet: 0)),
  (draft: true),
).enumerate() {
  if i > 0 { pagebreak() }
  flora-page(..opts)
  heading[My document]
  parbreak()
  text(13pt, lorem(70))
}
