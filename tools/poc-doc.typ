#let ancho = 21cm - 2.2cm - 5.6cm
#let bloque(arriba, abajo, it) = block(width: 100%, above: arriba, below: abajo, context [
  #metadata((p: here().page(), y: here().position().y.pt(), h: measure(block(width: ancho, it)).height.pt())) <flora>
  #it
])
#let flora = sys.inputs.at("flora", default: none)

#set page(
  paper: "a4",
  margin: (left: 2.2cm, right: 5.6cm, top: 2.5cm, bottom: 2.5cm),
  background: context if flora != none {
    image(flora + "/flora-" + str(here().page()) + ".pdf", width: 100%, height: 100%)
  },
)
#set text(lang: "es", size: 11pt)
#set par(justify: true)
#show par: it => bloque(1.2em, 1.2em, it)
#show heading: it => bloque(1.4em, 0.8em, it)

= Márgenes que crecen

#lorem(120)

#lorem(90)

#v(7cm)

#align(center, emph[Un claro reservado: la flora entra aquí.])

== Segunda sección

#lorem(150)

#lorem(180)

== Tercera sección

#lorem(200)

#lorem(160)
