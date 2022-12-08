# Allotment

This folder contains the code to position boxes on the screen, plus additional miscellaneous properties transmitted through styles. To understand this process see `leaf.md`. This document describes the code-organisaion rather than the principles.

## Top-level layout

* `collision` contains the bumping algorithm implementation.

* `containers` contains the container types for the layout tree, ie objects implementing anything non-terminal. There are different types for different ways of organising children: satk, overlay, bump etc, and also the root.

* `core` these are a few, small, pervasive types shared across this section and some used in the external api.

* `layout` implements the layout tree and its laying out alogirthm.

* `leafs` implements the leaf types: leafrequest, floatingleaf, anchoredleaf, etc.

* `style` implements the style tree.

## Collision

The main interface to the bumping algorithm is `bumpprocess.rs` which contains the type `BumpPersistent`. For details of this process, see `collision.md` in that directory. It is used almost exclusively by `container/bumper.rs` which is the layout node which supports bumping.

## Container and Layout

In `containers/` are the three main container types, `bumper`, `overlay`, and `stacker` which implement the three `ContainerAllotmentTypes`: to bump their contents, overlay their contents and stack them respectively.

```
             X
            /|\
           A B C

X is a OVERLAY               STACKER                  BUMPER
      +--------------+      +-------------+           +-----------------+
      |+------------+|      |+-----------+|           | +--------+      |
      || A,B & C all||      ||A          ||           | | A      |      |
      || over each  ||      ++-----------+|           | +-+----+-+----+ |
      || other      ||      ||B          ||           |   | B  | C    | |
      |+------------+|      |+-----------+|           |   +----+------+ |
      +--------------+      ||C          ||           +-----------------+
      eg fixed backgrounds  |+-----------+|           eg bumped genes
      and axes superimposed +-------------+
      on data              eg stacks of
                            transcripts
```

Also in here is `Root`, the speical top node of the layout tree and the cross-cutting traits and objects in `container`. See `layout.md` for details of this code and the processes involved after the initial overview from `leaf.md`.

The directory `layout` has the layout tree code itself (and various utility data types). Again, it is described in `layout.md`.

## Leafs

The vaious leaf types are implemented in `leafs`. See `leafs.md` for the meaning of the different leaf types. However, in summary, and in order of their transformation from "stlye script output" to "used for drawing":

`leafrequest` -- essentially merely a placeholder for some leaf mentioned in a style script. Scale invariant.

`floating` -- a box at some specific scale and with its internal bounds fully determined. Eventually, during the build process, it also acquires puzzle values for its eventual placing on the page.

`anchored` -- for some specific train (ie combination of carriages to be drawn on the screen next to each other) has a concrete value giving its offset on the screen (through resolving the puzzles in floating in the context of the given carriages).

`auxleaf` -- once the anchored co-ordinates have been used to transform the shapes, all that is left is an auxleaf containing miscellaneous odds-and-ends, such as z-index.

## Style trees

Layout trees need to look up styles in style trees. See `styletree.md` for details. They are created and built outside of the allotment folder and passed in during layout. They are used (ie lookups performed on them) during the `get_leaf` call in `ContainerOrLeaf`. It passes the relevent fragments of style to the constructors of the containers or leafs of the layout tree.

## Miscellaneous core objects

There are a few miscellaneous objects used throughout the layout code.

* `AllotmentName` -- this sinply wraps the string vector refering to some leaf. It calculates a hash on creation as it's regularly used as a key. There is also a special HashMap type to make looking up things by `AllotmentName` efficient.

* `RangeUsed` -- represents horizontal range in `ContentSize` for use in bumping, etc. Can have values `All` and `None` as well as specific intervals.

* `LeafRequestSource` -- used in the creation of shapes to get a `LeafRequest`. Makes sure the same object is returned for the same requested path by storing them in a map. Also contains the top-level code to drive these into `FloatingRequest` leafs (ie much of the top-level of the algorithms described in this file), in `to_floating_shapes()`.

* `FloatingCarriage` -- used to represent all the floating shapes in a carriage. Lazily calls `to_floating_shapes()` above, and also contains methos to generate anchored shapes and from them auxleaf shapes, in `unfloat_shapes`. THis object is the true top-level of the layout algorithm.

