# Eard files quick reference

Overview of common commands/values used in the eard programs to help with building genome browser tracks.
Also see [`registering-new-track.md`](/doc/registering-new-track.md)

## Styling system meta-language

Properties/values used in the style blocks in `style!("...")` macros (or `style()` calls).
Style blocks override previously set fields for matching container paths.

### Container properties
Parsed in [`containerstyle.rs`](/peregrine-data/src/allotment/style/containerstyle.rs)

- `system`: sets a coordinate system for child conainers/leafs
  - `tracking`: use genomic coordinates, scrolls with location
  - `content`: main content area (between side padding)
  - `left/right`: side paddding area
  - `tracking-window`: window coordinates (0..1), scrolls
  - `window` (default): absolute window coordinates (0..1)
- `type`: sets a container layout system (for child cotainers/leafs)
  - `stack` (default): children (tracks) are stacked vertically
  - `overlay`: content may overlap (`depth` sets paint order)
  - `bumper`: bumps content to prevent overlaps (variable track height)
  - `wall`: reserved allotment (acts as barrier in layout algorithm)
- `padding-top`/`padding-bottom`:
  - `<number>`: pixels added to top/bottom of the content in the container
- `min-height`:
  - `<number>`: minimum pixel height reserved for the content
- `priority`: content layout/allotment order (independent of `depth`)
  - `<integer>`: used for track/lane ordering (lowest value placed on top)
- `set-datum`:
  - `<identifier>`: records the container’s width (used for side padding, see below)
- `height-adjust`:
  - `tracking`: container height responds to tracking-space scaling logic
- `report`:
  - `<string>`: attach metadata to track container (sent to client-side when the track is enabled)

### Leaf / inheritable properties
From [`leafstyle.rs`](/peregrine-data/src/allotment/style/leafstyle.rs)

- `depth`: set layering/paint order
  - `<integer -128..127>`: used for background/blocks/title layering (higher value drawn later/on top)
- `system`: override container coordinate system
- `indent`:
  - `left` | `right` | `top` | `bottom`: align against that side
  - `none`: (default) no indentation
  - `datum(name)`: horizontal offset by side padding width (defined via `set-datum: name`)
- `bump-width`/`bump-height`:
  - `none`: exclude this leaf from horizontal/vertical bump (collision) calculations

---

## Drawing commands

### Coordinates & leafs
See [`shape.rs`](/libperegrine/src/shape.rs)

- `coord([position], [x_delta], [y_delta])`: registers a coordinate handle (point in canvas)
  - `position`: genomic position (or `0..1` if `window`/`content` coordinate system)
  - `x_delta`/`y_delta`: shift coordinate `<int>` pixels from position
- `leaf("path/to/leaf")`: creates a leaf (binds styles with matching paths)
- `empty([nw_coord], [se_coord], [leafs])`: reserves layout area without drawing (leaf anchoring)

### Colour & paint
See [`paint.rs`](/libperegrine/src/paint.rs)

- `colour!("#rgb")` / `colour!("name")`: registers a colour handle
- `paint_solid(colour_handle)`: registers paint for filled blocks
- `paint_hollow(colour_handle, width)`: paint for hollow blocks
  - with pixel `width` stroke line
- `paint_dotted(colour_a, colour_b, length, width, prop)`: paint for dashed/dotted lines
  - `length`: dash length for each colour
  - `width`: line width
  - `prop`: proportion (e.g 0.5) for pattern repeat/blend
- `paint_special("name", bool)`: used for maypole line
- `paint_metadata(key, [values], [templates])`:
  - binds metadata to a shape (used for focus tracks)
- `paint_setting(key, [values], [templates], hover_bool)`:
  - clickable hotspot to toggle a setting (used for transcript lozenges)
- `zmenu(variety_template, payload_template, hover_bool)`:
  - clickable hotspot for zmenu (sends zmenu payload to the client)
- `graph_type(height, colour_handle)`:
  - Plotter definition for wiggles (bar/line style with vertical scale).
- `pen(font_name, size, fgd_colour(s), bgd_colour(s))`:
  - Text pen; negative `size` attaches from right (mirroring).
  - Foreground/background colours can be single or sequences.

### Shapes

- `rectangle(nw_coord, se_coord, paint, leafs)`: draws a filled or stroked block
- `rectangle_join(nw, se, paint, leafs_a, leafs_b)`:
  - joint drawing to extra leafs (used for variant fishing lines)
- `running_rectangle(nw, se, end_bp, paint, leafs)`:
  - draws a block that stays on screen until `end_pos` visible (used for transcript lozenges)
- `wiggle(bp_left, bp_right, graph_type, [values], [full_values_bool], leaf)`:
  - draws a wiggle plot (used for gc track)
- `text(coords, pen, string_or_sequence, leafs)`:
  - Places static text at coordinate origin (extent from pen metrics).
- `running_text(nw_coord, se_coord, pen, strings, leafs)`:
  - used for gene labels (sticks with gene block until scrolled out of screen)
- `image(coords, image_name, leafs)`:
  - draws an image (from `backend-server/assets` dir), used for chevrons