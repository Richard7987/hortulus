#!/bin/sh
# Paints each case with the native build and compares it with tools/cases/expected.txt.
# UPDATE=1 rewrites the expected sums instead; do that only for an intended change in the drawing.
set -e
cd "$(dirname "$0")/.."
(cd plugin && cargo build --release --bin native 2>&1 | grep -E '^error' -A8) && exit 1
mkdir -p build/matrix
out=build/matrix/sums.txt
: >$out
c() {
  ./plugin/target/release/native "$2" "$3" build/matrix/$1 >/dev/null
  echo "$1 $(cat build/matrix/$1.body build/matrix/$1.res | sha256sum | cut -c1-16)" >>$out
}
S=tools/cases/a4.spec; H=tools/cases/header.spec; T=tools/cases/title.spec
c right $S "seed=3, layout=right, gilt=vector"
c split2 $H "seed=4, layout=split, gilt=vector"
c centered $H "seed=5, layout=centered, gilt=vector"
c surround $T "seed=6, layout=surround, gilt=vector"
c title $T "seed=8, layout=title, gilt=vector"
c spring $S "seed=9, colourway=spring, gilt=vector"
c winter $H "seed=10, colourway=winter, gilt=vector"
c ember $S "seed=11, colourway=ember, gilt=vector"
c random $S "seed=12, palette={hero=random, blooms=random, leaves=teal}, spin=3, gilt=vector"
c visitors $S "seed=13, visitors=3, gilt=vector"
c visitors2 $H "seed=14, visitors=5, colourway=summer, gilt=vector"
c mixlily $S "seed=15, mix={lily=3, umbel=2, gyps=2, ghost=2}, gilt=vector"
c density $S "seed=16, density=0.5, gilt=vector"
c dense $S "seed=17, density=1.8, gilt=vector"
c noleaf $S "seed=18, mix={leaves=0, leaflets=0, curls=2}, gilt=vector"
c raster $S "seed=19"
for k in night sun moon constellation planet armillary scrolled strapwork; do
  z="night=0, sun=0, moon=0, constellation=0, planet=0, armillary=0, scrolled=0, strapwork=0"
  c cart-$k $S "seed=21, ornaments={$z, $k=5}, gilt=vector"
done
for k in compass rosette sunburst quatrefoil; do
  c head-$k $H "seed=22, ornaments={compass=0, rosette=0, sunburst=0, quatrefoil=0, $k=5}, gilt=vector"
done
for k in knot medallion zodiac_pendant; do
  c pend-$k $S "seed=23, ornaments={knot=0, medallion=0, zodiac_pendant=0, $k=5}, gilt=vector"
done
for k in sun_crest compass_rose armillary_crest zodiac wreath; do
  c crest-$k $T "seed=24, layout=title, ornaments={sun_crest=0, compass_rose=0, armillary_crest=0, zodiac=0, wreath=0, $k=5}, gilt=vector"
done
for k in swash fleuron_rule headpiece copperplate swelled_rule fleuronnee rinceau; do
  for s in 25 26 27; do
    c flo-$k-$s $T "seed=$s, layout=title, ornaments={swash=0, fleuron_rule=0, headpiece=0, copperplate=0, swelled_rule=0, fleuronnee=0, rinceau=0, $k=5}, gilt=vector"
  done
done
for k in cul_de_lampe hedera garland; do
  c tail-$k $T "seed=28, layout=title, ornaments={cul_de_lampe=0, hedera=0, garland=0, $k=5}, gilt=vector"
done
for k in bracket fan corner_leaf acanthus_corner cornerpiece pen_corner; do
  c corn-$k $T "seed=29, layout=title, ornaments={bracket=0, fan=0, corner_leaf=0, acanthus_corner=0, cornerpiece=0, pen_corner=0, $k=5}, gilt=vector"
done
for k in gilt_cross gilt_rosette gilt_dotted gilt_lozenge gilt_star gilt_crescent gilt_trefoil; do
  c gilt-$k $S "seed=30, ornaments={gilt_cross=0, gilt_rosette=0, gilt_dotted=0, gilt_lozenge=0, gilt_star=0, gilt_crescent=0, gilt_trefoil=0, $k=5}, gilt=vector"
done
for l in left right centered split surround; do
  c natural-$l $H "seed=31, layout=$l, growth=natural, gilt=vector"
done
c natural-title $T "seed=32, layout=title, growth=natural, colourway=plum, gilt=vector"
c natural-dense $S "seed=33, growth=natural, density=1.8, gilt=vector"

if [ -n "$UPDATE" ]; then
  cp $out tools/cases/expected.txt
  echo "$(wc -l <$out) cases written to tools/cases/expected.txt"
  exit 0
fi
total=$(wc -l <$out)
bad=$(diff tools/cases/expected.txt $out | grep '^>' | cut -d' ' -f2)
if [ -z "$bad" ]; then
  echo "$total/$total equal"
else
  echo "differ:" $bad
  exit 1
fi
