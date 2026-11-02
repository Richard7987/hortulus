#import "/package/lib.typ": *
#set page(paper: "a4", margin: (left: 2.2cm, right: 5.6cm, y: 2.5cm), header: align(right)[Florilegio])
#set text(lang: "es", size: 11pt)
#set par(justify: true, first-line-indent: 1em)
#show: flora.with(seed: 7, layout: "split", colourway: "spring", gilt-dpi: 300, ..sys.inputs.at("extra", default: (:)))

= Márgenes que crecen

#lorem(90)

#lorem(60)

#v(6cm)

#align(center, emph[Un claro reservado: la flora entra aquí.])

== Listas y más

- Primero un elemento de lista.
- Luego otro elemento, algo más largo para que ocupe sitio en la línea.
- Y un tercero.

+ Numerado uno
+ Numerado dos

#table(columns: 3, [a], [b], [c], [1], [2], [3])

$ integral_0^1 x^2 dif x = 1/3 $

```rust
fn main() { println!("hola"); }
```

#figure(rect(width: 8cm, height: 3cm, fill: luma(230)), caption: [Una figura.])

#lorem(240)

#flora-page(colourway: "autumn")

== Otra página

#lorem(120)
