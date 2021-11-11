# 0.0.11

  * Remove lots of unnecessary key bindings interfering with browser chrome
  * Reformat zmenu payloads

# 0.0.10

 * Backend/frontend versioning support for multiple egs files, depending on FE capabilities.

# 0.0.9

 * reliably show sequence letters at zoomed-in level
 * show red-dotted line even at most zoomed-in level
 * correct swatch colour
 * corrext swatch size
 * correct font size in track category

# 0.0.8

 * Cleaner backand request/response API on frontend
 * Backend errors no longer cause CORS knockon errors
 * GC track fixes:
   * wrong colour;
   * horiz bar missing;
   * incorrect spikiness;
   * mini-blank bug;
   * incorrect at most-zoomed-in level;
   * missing data blanks.

# 0.0.7

 * cleaner use of WebGL
 * new target location mechanism.

# 0.0.6

 * more intelligent handling of failed backends

# 0.0.5

 * bugfix for vertical dotted lines (broken by 0.0.4)
 * display dotted red lines
 * don't show dotted lines when focus track is off.

# 0.0.4

* fix lhs track category track-on/off bug
* fix lhs swatch track-on/off bug

# 0.0.3

* fix tracks-drawn-in-wrong-place bug
* add track category letters
* add "bp" edges to ruler and correct overlapping on rhs
* blue swatch for focus track
* Bug: lhs doesn't interact well with turning tracks off (yet)

# 0.0.2

* bottom ruler in the correct place, 
* white left and right sides. (blank)
* Bug: in some positions the tracks are sometimes drawn in the wrong place
* still a bit slow on wheat (at least on the s3 server)
