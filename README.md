# Hortulus

Watercolour flora and gilt ornaments grown in the margins of Typst documents.

**Version 0.1.0.** [Release notes](https://github.com/Richard7987/hortulus/releases/tag/v0.1.0) ·
[Full documentation](package/README.md) ·
[Typst Universe submission](https://github.com/typst/packages/pull/6288) (in review)

<img src="package/images/gallery.png" alt="Four pages: a right margin, a crowned title page, a wreath around the text, and flora in both margins" width="100%">

## Quick start

Until the package is on Typst Universe, copy `package/` next to your document as `hortulus/`:

```typ
#import "hortulus/lib.typ": flora

#set page(margin: (left: 2.5cm, right: 5.5cm, y: 2.5cm))
#show: flora.with(layout: "right", colourway: "spring", seed: 7)

= My document
#lorem(200)
```

Once it is published, import it as `@preview/hortulus:0.1.0`.

## Gallery

Up close: the blooms are vector paths painted like watercolour, and the gold is computed per pixel.

<img src="package/images/detail.jpg" alt="Close-up of a title page: two blue blooms on either side of a gilt laurel crest" width="100%">

The eight colourways: `spring`, `summer`, `autumn` and `winter` above; `ember`, `plum`, `rose` and `original` below.

<img src="package/images/colourways.jpg" alt="Eight pages with the same layout, one in each colourway" width="100%">

The same page with other options. Above: `density: 0.4`, `density: 2`, `visitors: 4` and `mix: (lily: 3, bloom: 0, umbel: 0)`. Below: `palette` with one ink, `palette` with two inks, `ornaments: (moon: 5, night: 0, sun: 0, planet: 0)` and `draft: true`.

<img src="package/images/tuning.jpg" alt="Eight pages showing density, visitors, mix, palette, ornaments and draft" width="100%">

The plant grows around what the page holds: a gap in the text, a table and an equation, a floating figure, and two columns with `layout: "split"`.

<img src="package/images/content.jpg" alt="Four pages where the flora fills a gap, surrounds a table and an equation, stops at a floating figure, and follows two columns" width="100%">

## Repository

- `package/`: the Typst package, as published.
- `plugin/`: the Rust source of the package's WebAssembly plugin.
- `reference/`: the Lua generator of [tikz-flora](https://github.com/wn-mitch/tikz-flora) (MIT),
  which the plugin ports and is checked against.
- `tools/`: the build scripts and the tests.

## Building the plugin

You need a Rust toolchain with the `wasm32-unknown-unknown` target, and `wasm-opt` from Binaryen.

```sh
RUSTUP_HOME=... CARGO_HOME=... tools/build_wasm.sh   # see the script for the paths it sets
cp build/hortulus.wasm package/hortulus.wasm
```

## Rebuilding the images

With `typst` and ImageMagick on your path, `tools/build_gallery.sh` paints the gallery from
`tools/gallery/` into `package/images/`.

## Checking against the reference

With `texlua` on your path and the native build in place (`cargo build --release` in `plugin/`),
`tools/run_matrix.sh` paints each case with both generators and compares the output byte for byte.

## License

MIT. See `LICENSE`. `reference/` keeps its own license.
