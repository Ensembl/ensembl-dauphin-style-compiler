# Eard files quick reference

Overview of common commands/values used in the eard programs to help with building genome browser tracks.
Also see [`registering-new-track.md`](/doc/registering-new-track.md)

## Styling system meta-language

Properties/values used in the style blocks in `style!("...")` macros (or `style()` calls).
Style blocks override previously set fields for matching container paths.
Base styles for all tracks are defined in [`track-style.eard`](egs/v16/common/track-style.eard).

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

- `coord(position[int], x_delta[int], y_delta[int])`: registers a coordinate handle (point in canvas)
  - `position`: genomic position (or `0..1` if `window`/`content` coordinate system)
  - `x_delta`/`y_delta`: shift the coordinate `<int>` pixels from `position`
- `leaf(path_str)`: creates a leaf (binds styles with matching paths)
- `empty(nw_coord[], se_coord[], leaf)`: reserves layout area without drawing (leaf anchoring)

### Colour & paint
See [`paint.rs`](/libperegrine/src/paint.rs)

- `colour!("#rgb")` / `colour!("name")`: registers a colour handle
- `paint_solid(colour)`: registers paint for filled blocks
- `paint_hollow(colour, stroke_width)`: paint for hollow blocks (stroke line width in pixels)
- `paint_dotted(colour_a, colour_b, length, width, prop)`: paint for dashed/dotted lines
  - `length`: dash length for each colour (px)
  - `width`: line width (px)
  - `prop`: proportion (e.g `0.5`) for pattern repeat/blend
- `paint_special(name_str, bool)`: used for ruler maypole line (name = "maypole")
- `paint_metadata(key, values[], eoe_templates[])`:
  - binds metadata to a shape (used for focus tracks)
- `paint_setting(key, values[], eoe_templates[], hover_bool)`:
  - clickable hotspot to toggle a setting (used for transcript lozenges)
- `zmenu(variety_template, payload_template, hover_bool)`:
  - clickable hotspot for zmenu (sends zmenu payload to the client)
- `graph_type(height, colour)`: parameters for wiggle graph
- `pen(font_str, size_int, fgd_colours[], bgd_colours[])`: paint handle for text
  - negative `size` attaches from right (mirroring)

### Shapes

- `rectangle(nw_coord, se_coord, paint, leafs[])`: draws a filled or stroked block
- `rectangle_join(nw_coord, se_coord, paint, leafs_a[], leafs_b[])`:
  - joint drawing to extra leafs (used for variant track fishing lines)
- `running_rectangle(nw_coord, se_coord, end_bp, paint, leafs[])`:
  - draws a block that stays on screen until `end_bp` is visible (used for transcript lozenges)
- `wiggle(start_bp, end_bp, graph_type, y_values[float], include_values[bool], leaf)`:
  - draws a wiggle plot (used for gc track)
- `text(coords, pen, text[str], leafs[])`: draws a static text
- `running_text(nw_coord, se_coord, pen, strings, leafs)`:
  - used for gene labels (sticks with gene block until scrolled out of screen)
- `image(coords, image_name, leafs)`:
  - draws an image (from `backend-server/assets` dir), used for chevrons

## Built-ins

See [peregrine-eard](https://github.com/Ensembl/peregrine-eard/blob/main/docs/library-ref-source.txt) doc to reference built-in functions and operators (e.g. `print()`,`len()`, `eoe` templates).