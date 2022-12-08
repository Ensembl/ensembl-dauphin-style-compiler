# Hotspots

ZMenu intersection must be stored efficiently not so much for clicks (which can afford a millisecond or two) but for cursor change, which happens regularly as you move around the screen.

## Zoning algorithm

### Overview

For speed, the screen is divided into zones (around a hundred). When adding a zmenu all zones which it might enter are marked with this zmenu. Post-filtering is the applied to eliminate zmenus which don't intersect after all. So on move only a single zone is examined.

### Coordinate systems

ZMenus can be attached to a number of coordinate systems which differ in:

1. whether they are "document"-like or "window"-like (in both the x and y axis);
2. whether they are "negative" (ie from bottom/right of screen);
3. whether they are rotated.

Individual values also differ in whether they are positive or negative in sense. Both negative co-ordinates of individual values and of whole boxes are both supported to allow for accurate playing-field squeezing. (eg the left and right buffers are discounted when jumping to a region).

However, the vast majority of zmenus will be in positive document/document co-ordinate systems (things on the page), or positive window/document co-ordinate systems (things in vertical sidebars).

To keep the edge cases simple, only these two cases use the zoning system. Other zmenus are put into a "remainders" list which is checked linearly each time.

### positive document/document zoning

Document/document zoning happens on a per-carriage basis. Vertical zoning uses a fixed pixel size for each zone, horizontal zoning divides each drawing into a fixed number of zones.

### positive window/document zoning

Window/document zoning is a singleton. Vertical zoning again uses a fixed pixel size for each zone. Horizontal zoning is not attempted. There are likely to be a combination of left-left, left-right, and right-right objects (left sidebar, track width, and right sidebar respectively), however very little congestion on the horizontal axis, it therefore makes sense just to vertically stripe zmenus.

## Types

* A `Hotspot` is a zmenu or otehr hotspot in abstract, independent of its co-ordinates. 

* A `HotspotUnscaledEntryDetails` is 