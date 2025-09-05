# Eard files quick reference

Overview of common commands/values used in the eard programs to help with building genome browser tracks.
Also see [registering-new-track.md](/doc/registering-new-track.md)

## Styling system meta-language

Properties/values used in the style blocks in `style!("...")` macros (or `style()` calls).
Style blocks override previously set fields for matching container paths.

### Container properties
Parsed in [containerstyle.rs](/peregrine-data/src/allotment/style/containerstyle.rs)

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
- `priority`: set layout/allotment order (independent of `depth`)
  - `<integer>`: used for track order and element order (title/bacgkound/content)
- `set-datum`:
  - `<identifier>`: records the container’s width (used for side padding, see below)
- `height-adjust`:
  - `tracking`: container height responds to tracking-space scaling logic
- `report`:
  - `<string>`: attach metadata to track container (sent to client-side when the track is enabled)

### Leaf / inheritable properties
From (leafstyle.rs)[/peregrine-data/src/allotment/style/leafstyle.rs]

- `depth`: set z-order / paint order
  - `<integer -128..127>`:  leaf with larger value drawn later / on top
- `system`: override container coordinate system
- `indent`:
  - `left` | `right` | `top` | `bottom`: align against that side
  - `none`: (default) no indentation
  - `datum(name)`: horizontal offset by side padding width (defined via `set-datum: name`)
- `bump-width`/`bump-height`:
  - `none`: exclude this leaf from horizontal/vertical bump (collision) calculations

