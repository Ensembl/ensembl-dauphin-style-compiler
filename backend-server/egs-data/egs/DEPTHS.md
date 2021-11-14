# Depths and opacity, fading

Depths are used in the genome browser so that one visual element can obscure another without a strict requirement
of rendering order. This is particularly important as some allotments overlap others but also because multiple
regions could draw an element at their boundary, and similar considerations.

Sadly, depth and transparency interact poorly in WebGL (which reflects restrictions in the underlying graphics cards
and can't be easily coded away). In essence, depth is an all-or-nothing test as to whether an item is closer to
anything drawn so far (and if not to mask it entirely). Should it pass the depth test, its colour is mixed with what has
been rendered already (either at a lower depth or the background). It acquires its colour _at the moment it is added_.
The practical effect of these algorithms is that you cannot "slip a shape behind" ones already drawn and expect 
any partially-transparent elements closer to the user which have already been drawn to reflect that change.

Ideally, items are rendered from back to front, but that's not possible to do efficiently in our model. A major effect
isvthat when fading between elements of different scales, the first group of elements to be rendered acquire very 
close to the colour of the background when first, faintly rendered. All members of the second group of elements to be
rendered at a depth less than that of the first group are then obscured completely immediately at the start of the
fade. Typically this manifests as both rapid flickers of white as each is rendered accompanied by slow over-dominance
of white throughout the fade.

The strategy taken by the genome browser is based on the idea that:

1. during transitions items are drawn zoomed-in to zoomed-out. 
2. zoomed-out is drawn close to the user.

Zoomed-in items tend to have more detailed realisations compared to their zoomed-out versions, which are often just
blocks. In particular they tend to have backgrounds and "curtains", blocks which fade into more detail, as the 
unveiling of an item. If the curtains are drawn close to the user (with the backgrounds at identical positions at
all scales), as the curtains fade they reveal the detail drawn earlier (further from the viewer). Note that we are
approximating the "draw furthest away first" strategy by doing this.

# Depth values

Depths are represented by signed 8-bits, -128 to 127, though it would be inadvisable to use numbers beyond +/- 100 to
avoid overflow bugs. For the simplest tracks, and for uncomplicated parts of other tracks (eg labels), the data should
sit at 0. Broad ranges are reserved for background, chrome, etc, which are then further divided.

* -50 -- -10 -- track backgrounds
* -9  -- +49  -- track contents
* +50 -- +100 -- overlaying chrome

#  Tracks (common)

* -10 -- chevrons, etc; track labels
* 0 -- regular contents (simple tracks); labels

# Gene track

* -3 -- transcript view: dotted gene line
* -2 -- transcript view: solid gene line
* -1 -- transcript view: exon-blanking rectangles
* 0 -- seqence view: hollow squares, solid squares, hollow letters
* +1 -- seqence view: solid letters
* +2 -- transcript view: exon hollow box
* +3 -- transcript view: exon solid box
* +4 -- gene view: solig gene rectangles

# Overlaying chrome

* +54 -- vertical padding ruler should go over (rhs)
* +55 -- ruler
* +56 -- vertical padding ruler should go under (lhs)
* +58 -- "bp" at top and bottom left
* +60 -- vertical padding contents (category letters)
