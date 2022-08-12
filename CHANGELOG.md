# 0.5.0
  * Drop support for 0.3.x clients (all versions prior to 9)

# 0.4.4
  * Features
    * "Flagtop" track for showing focus region endpoints
    * special, dark lhs chevrons
  * Feature-enabling functionality
    * text alignable against right edge (for flagtop)
  * Bug-fixes:
    * don't show any zmenus when over dotted lines
    * variation track name per side panel
    * Remove dashes from track names
    * large gene truncation issue (seen in MAPK10)
    * lhs white ground data showing through bug
    * stop zmenus forcing full gene size when fewer transcripts shown
    * Doubled letter rhs bug
    * rhs frame layering bug
    * remove hyphen from focus track.
  * Build process
    * fix handling of dpr-force parameter when dpr-forcing disabled

# 0.4.3
  * Bug-fixes:
    * Fix chevron tangle bug
    * Fix gene labels drawn on wrong transcripts
    * Fix zmenus not including type payload
    * Don't report trivial changes and rounding errors as location events on click
    * New track names
    * Use item object as focus track name
    * Fix track ordering
    * Fix dotted line height bug
    * Fix dotted line device-pixel-ratio bug
  * Build process
    * Allow dockerised integration to checkout ensembl-client on given branch
    * Compile time ability to override device-pixel-ratio for testing

# 0.4.2
  * Separate focus egs file to allow always shown in transcript view.
  * Factoring egs stuff shared between tracks
  * Impl of "don't know" state for eye icons
  * Fix label spacing between gene and transcript labels
  * Left-sticky labels

# 0.4.1
  * Fix reporting of boxes from flanks (reported in 0.4.0)
  * Multiple transcript update from genes
  * Visual fixes
    * move labels to bottom of genes
    * vertical alignment between transcript and sequence.
    * flip boxed and unboxed UTR/intron; fix split UTR rendering bug.
    * fix utr box colour bug
    * can't turn first tr off bug
    * show tr labels on zoomed view
    * make tr labels independent of gn labels
    * dotted lines at zoomed scale

# 0.4.0
  * Failed unload bug fix (again)
  * Multiple transcript reporting
    * Flexible data-types in reporting
    * Unit testing
    * Metadata report object
  * Visual fixes
    * make lozenges blue and round corners
    * reduce overwhelming chevron opacity to a point people don't notice it in advance of design review
    * Lato -> Plex
    * Correct, ultra-fine line thickness
    * Open out focus track somewhat
    * Fix wonky blue lhs swatch so doesn't become too noticeable in advance of design review
    * Pick off some inter-scale spacing-fix tasks: no bumping labelless view
    * remove endcaps for focus genes
    * em-dashes not dash-dash.
    * unboxed letters for UTRs.
    * grey sequence
    * Don't show letters until more zoomed in.

# 0.3.2
  * Failed unload bugfix

# 0.3.1
  * Fix image size bug (chevrons)
  * Multiple transcripts
    * show transcript ids (incorrectly laid out)
    * "several" flipping for lozenges (disabled in release)
  * Speed improvements
    * In-code documetnation
    * Performance sampler to test speed issues
    * Improved debugging of train transitions
    * Improve anticipation algorithm
    * Bugfixes (speed) in train code
    * Refactor inout code (tidy and speed up)
    * Flashes-of-white bugfix
    * Refactor railway code
    * Optimise style-compiler programs
    * Fix unit tests
    * Preload wgsl
    * Fix resizer
    * EachOrEvery fixes for indexed vectors
    * Replace mask with alpha in 2d backing bitmaps

# 0.3.0
  * Gene bumping
    * Add "functional" puzzle systm for inter-carriage dependencies
    * Full box model for vertical positioning
    * Style model
    * Accompanying style-compiler program changes
  * Multiple transcripts on screen
    * "Several" transcripts mode via `["track",X,"several"]` flag
    * Support for on-screen buttons (not used this release)
    * Accopanying style-compiler program changes
  * New "empty" shape which occupies space but has no content
  * Support for scope flags in BE communication. 
  * Memcached backend flushing
    * alwaysflush in dev mode
    * take egs version into account in production mode
  * Tidy & Refactor
    * Carriage-positioning code
    * Each/Every repetition code
    * spacebase coordinate sytem
    * spectres for marching-ants zoom
  * Bug fixes
    * Fix stall bug in scheduler when under load
    * Fix screen resize bug
    * Fix various styling bugs in sequence view
  * NEW BUGS / REVERSIONS (to be removed in 0.3.1):
    * some alignment bugs when transitioning scales make it look dodgy
    * some flicker during scaling
    * slow sequence views

# 0.2.1
  * Build system improvements
    * Document it
    * Present previous options for confirmation to allow easy repeat builds
    * Change DOckerfile in light of npm scope changes
  * Reduce fuzziness
    * Take devicepixelratio into account on both webgl and bitmap canvases
    * Fix dotted lines in various minor ways
    * Add scale option to pngs
    * Fix scaling of canvas in integration (also bottom ruler reappears).

# 0.2.0
  * Round sizes correctly in observer to avoid excessive CPU usage
  * Don't crash if window size becomes zero
  * New "variety" key to reduce zmenu ambiguity.
  * Deduplicate varieties
  * Update current style compiler programs to use "variety key" (BE version 6->7, code version 0.1.x->0.2.x)
  * Allow an element to be passed in the config instead of an element id (if desired)
  * Replace ad-hoc approaches to logging verbosity on console with single, unified and configurable approach
  * Build system work:
    * Use dockerignore to reduce build time of docekr builds
    * Use docker buildkit for extra features
    * Menuize build scripts
    * Bust docker cacheing to pick up github changes, where requested
  * Code quality work in callbacks:
    * Simpler scheduler
    * New, self-contained classes which manage unloading correctly for
      * timer
      * requestanimationframe
      * custom events

# 0.1.4
  * Remove annoying "double flash" on gene move
  * Additional key in zmenu payload to link genes and transcripts
  * Move data release on in standalone browser
  * Fix accidentally-changed library hash

# 0.1.3
  * Fix various scrolling bugs
    * Bug-fixes in chromosome switching
    * Enforce size/position limits in more places
    * Use van Wijk and Nuij's hyperbolic space algorithm for moves and pans rather than pile of buggy heuistics
    * Use fade not animate for very long moves even if on same stick.
  * Misc draw speed bugfixes and instrumentation
  * Fixed version strings so that they survive CI/CD

# 0.1.2
  * Fix default tack order
  * Cleanly handle possibility of 404s (and other unexpected status codes) from BE
  * Cleanly fail on jump target not found

# 0.1.1
  * Reduce unnecessary draw groups for efficiency and code cleanliness
  * Milestone trains for smooth scrolling at speed
  * Fix anticipation code to make "lo" more effective following metrics analysis
  * Fix priority inversion stalling focus change instructions
  * Remove accidentally-retained console.log()s

# 0.1.0
  * Bugfixes
    * Report "bad stick" immediately, don't retry
    * Chromosome switch bug (wrong size used during switching)
  * Reduce technical debt to allow speed improvements
    * Rational use of depths: prevents layering bugs
    * Start recording metrics for WebGL buffers
    * Rewrite "railway" code, cleaner and more rational
    * Priority fixes
    * Rationalised and co-ordinated co-ordinate systems
  * Code tidying and improvements for large chromosomes
    * Less fuzzy display of very zoomed-out chromosomes
    * Manage absence of wheat contig file gracefully
    * Reduced reliance on spot colours
    * Tweak number of levels requiring data fetch
    * Shrink and rationalise buffer usage
    * Allow async even in webgl code
  * Increased protocol version to 6
    * Verified 0.0.12 works with 0.1.0 backend
  * REGRESSION: display juddery (already fixed in 0.1.1)

# 0.0.12

  * Select correct designated transcript in backend
  * (Hopefully) fix (at least reduce) jumping track bug.

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
