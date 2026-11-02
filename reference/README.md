# tikz-flora

Watercolour flora grown in the margins of LuaLaTeX documents. Every page
grows its own plant around the text it holds: stems, leaves, blooms, sprays,
gilt, and if you like, bees, butterflies, birds and snails. Everything is
computed in Lua and drawn through PGF, with no images.

Site, live mixer and full guide: **https://flora.alpacasoft.dev**

## Origins

This project would not exist without two others:

- The r/LaTeX thread
  [How do you make these flowers in LaTeX?](https://old.reddit.com/r/LaTeX/comments/1xixznw/how_do_you_make_these_flowers_in_latex/),
  whose photographed page is the reference this renderer is measured
  against.
- Kevin Raymond's [florilegium](https://github.com/kevinraymond/florilegium)
  (MIT), whose grown margins, watch-it-grow animation and visitors this
  package follows.

The ink colours are Pilot iroshizuku ramps measured by
[madoromi](https://madoromi.moonwit.ch/#palettes/colours). The growth follows
Wong, Zongker and Salesin,
[Computer-Generated Floral Ornament](https://grail.cs.washington.edu/projects/ornament/ornament.pdf)
(SIGGRAPH 1998).

tikz-flora was written with Claude Code, Anthropic's coding agent. The art
direction, the tuning against the reference and the choice of what to keep
were done by hand.

## Setup

You need a TeX distribution with **LuaLaTeX** (TeX Live 2023 or later,
MiKTeX, or TinyTeX). Check with `lualatex --version`. The example also uses
`lipsum` (`tlmgr install lipsum` on TinyTeX).

1. Download `tikzflora.zip` from the site, or build it with `just dist`.
2. Unzip it. It holds `tikzflora.sty`, `tikzflora-boot.lua`, the generator
   in `lua/flora/`, and `example.tex`.
3. Put your document next to `tikzflora.sty` (or copy the `.sty`, the boot
   file and `lua/` next to your document).
4. Compile with LuaLaTeX: `latexmk example` (the included `latexmkrc`
   selects LuaLaTeX), or `lualatex example` twice.

To install it for every document instead, copy the three items into
`TEXMFHOME/tex/lualatex/tikzflora/` (`kpsewhich -var-value TEXMFHOME` prints
the path).

**Overleaf:** New Project, Upload Project, choose `tikzflora.zip`. The
`latexmkrc` in the zip sets the compiler to LuaLaTeX; otherwise choose it
under Menu, Compiler.

A minimal document:

```latex
\documentclass{article}
\usepackage[left=5.5cm,right=2cm]{geometry} % room for the flora
\usepackage{tikzflora}
\florasetup{seed=7}
\begin{document}
Text.
\end{document}
```

## Settings

`\florasetup{...}` sets keys for the rest of the document, from the page it
is on. `\florathispage{...}` sets them for one page only. Mistakes stop the
compile with a message naming the key.

| Key | Values | Effect |
|---|---|---|
| `layout` | `left`, `right`, `centered`, `split`, `title`, `surround` | Where the flora grows (below) |
| `seed` | a number, or `random` | Which garden; each page varies it |
| `mix` | `{kind=weight, ...}` | How much of each kind: 1 usual, 0 none, 2 about double |
| `density` | 0 to 10 | Fuller (above 1) or airier (below 1) |
| `visitors` | 0 to 10 | The weight of every visitor at once |
| `palette` | `{part=ink, ...}` | What paints the `hero`, `blooms`, `leaves`, `leaflets`, `stems`, `sprays`, `ghost`, `cartouche` |
| `colourway` | `spring`, `summer`, `autumn` (or `fall`), `winter`, `ember`, `plum`, `rose`, `original` | A whole palette at once; a `palette` after it changes single parts |
| `spin` | a whole number | Picks other inks for parts set to `random`; the shapes stay |
| `ornaments` | `{kind=weight, ...}` | How often each ornament kind is picked for its place; 0 never |
| `gilt` | `raster`, `vector` | Ornament gold computed per pixel as images (the default), or built of flat-coloured strips |
| `giltdpi` | 50 to 1200 | Resolution of raster gilt; 600 by default, lower compiles faster |
| `giltenv` | `studio`, `mirror` | The look of raster gilt: a studio window over dark ground (default), or the vector relief's sky and horizon |
| `giltlook` | `{key=value, ...}` | Changes keys of that look, e.g. `giltlook={horizon=0.1, grain=0}`: light, horizon, sky, ground, highlight, grain, sparkle, shadow and more (`lua/flora/gilt_look.lua` lists every key and its range) |

Mix kinds: `bloom`, `soft`, `lily`, `ghost`, `umbel`, `cyme`
(forget-me-nots), `gyps` (baby's breath), `leaves`, `leaflets`, `sprays`,
`curls`, `density`, `bee`, `butterfly`, `bird`, `snail`. Visitors are off
unless you give them weight.

Palette values are `look` (the default gradients), an ink group (`blue`,
`blue-grey`, `teal`, `green`, `violet`, `warm`, `earth`, `gold`) or one ink
by name, such as `kon-peki` (all of them are in `lua/flora/inks.lua`).
`random` paints that part in one ink picked per page. The pick has its own
random stream, so changing `spin` changes only the colours: every stem,
leaf and bloom stays where it was.

`sprays` paints the forget-me-nots, baby's breath and umbels, `ghost` the
pale ghost bloom, and `cartouche` the sky in the star cartouche. A non-blue
ink also repaints the rims, grain and contours of the part it paints, so a
page with every part set keeps only a few navy pen lines.

Ornaments fill places on the page: the `cartouche` panel, the `header`
mark on the running-head rule, the `pendant` in the foot's corner, the
larger `gilt` in the foliage's gaps, and on title pages a `crest` where the
crown meets, a `flourish` under the title, a `tailpiece` under the last
line and a piece in each foot `corner`. Each place picks one kind by
weight, so `ornaments={moon=3, night=0}` favours the crescent moon
cartouche and drops the night sky; setting every kind of a place to 0
leaves it empty. `just gallery` (or `\floragallery` in a `tikzpicture`)
draws every kind with the name to weigh it by. The kinds:

- cartouche: `night`, `sun`, `moon`, `constellation`, `planet`,
  `armillary`, `scrolled`, `strapwork`
- header: `compass`, `rosette`, `sunburst`, `quatrefoil`
- pendant: `knot`, `medallion`, `zodiac_pendant`
- crest: `sun_crest`, `compass_rose`, `armillary_crest`, `zodiac`, `wreath`
- flourish: `swash`, `fleuron_rule`, `headpiece`
- tailpiece: `cul_de_lampe`, `hedera`, `garland`
- corner: `bracket`, `fan`, `corner_leaf`
- gilt: `gilt_cross`, `gilt_rosette`, `gilt_dotted`, `gilt_lozenge`,
  `gilt_star`, `gilt_crescent`, `gilt_trefoil`

A new kind is a Lua file that calls `register` in
`lua/flora/ornaments.lua` (which explains the fields) and is listed in its
`MODULES`; the gallery and the tests pick it up, and `tests/test_ornaments.lua`
holds it to its box.

```latex
\florasetup{layout=split, visitors=0.5, mix={lily=2, umbel=0},
  palette={hero=murasaki-shikibu, blooms=warm, leaves=teal}}
```

### Layouts

- `left`: the left margin (the default).
- `right`: the right margin.
- `centered`: both side margins, with two different plants.
- `split`: the outer margin of a two-sided document. It is on the right of
  odd pages and the left of even ones.
- `title`: a crown over a title. Wrap the title in
  `\begin{floratitle} ... \end{floratitle}`; without it the first block
  of lines is crowned.
- `surround`: a wreath round the text block, for title pages and contents.

The flora fills whatever margins the page has, so give it room with
`geometry` (about 5 cm for a side margin, or 3 cm all round for
`surround`).

It also grows into gaps in the text. A gap between lines taller than about
three lines (40 pt), such as a `\vspace` left for a figure, opens across
the text's full width and the flora reaches into it. The space below a
short last page fills the same way.

`tex/lorem.tex` (`example.tex` in the zip) uses each of these, with a
comment at every setting.

## Working on tikz-flora

The repository uses [just](https://github.com/casey/just). These recipes
add the TinyTeX binaries to `PATH`:

- `just build NAME [SEED]`: compile `tex/NAME.tex` to `build/` with a PNG
  preview. `just build lorem` builds the example; `just build reference`
  builds a page laid out like the reference photo, which the tuner uses.
- `just grow NAME SEED PAGE`: build `build/grow-NAME-PAGE.mp4` and `.gif`
  of a page as it grows. It needs ffmpeg, poppler (`pdftocairo`) and
  ImageMagick.
- `just gallery`: draw every ornament kind, labelled, to
  `build/gallery.pdf` (`tex/gallery.tex`).
- `just dist`: build `build/tikzflora.zip`. `just test-dist` compiles the
  example from the unpacked zip alone.
- `just test`: run the Lua and Python unit tests. `just test-tex` and
  `just test-preview` check against real compiles.
- `just site-bake`, `just site-dev`, `just test-site`, `just site-deploy`:
  build, test and publish the site. `site/README.md` explains them.
- `just tuner`: run a local web app with a slider for every setting in
  `lua/flora/tune.lua`, drawing `tex/reference.tex` beside the de-warped
  reference photo when `tex/reference/rectified.png` is present (it is not
  tracked). It needs Python 3 with numpy and Pillow, and Node.

## Licence

MIT. See `LICENSE`.
