# The Layout Algorithm

This file describes the practical code aspects of the layout algorithm. See `allotment.md` for how this fits into the code more broadly, and `leaf.md` for a higher-level overview of the whole process. Without reading at least the latter this document will seem very mysterious.

# General overview

Layout handles the key second and third stages of the layout algorithm: the generation of the layout tree and the recursive algorithm which runs through this tree, giving offsets and heights to leafs.

The layout tree is built by `build_layout_tree()` in `layouttree.rs`. This returns the root of the tree and a map from the AllotmentName (which is known to LeafRequest) to the newly created FloatingLeaf within the tree just created.

This output is then passed to the `full_build` method of the `Root` container, which runs the up-and-down algorithm on the tree, also generating any correlations for this carriage (see other documents for the correlation system). Finally, all shapes are mapped from their LeafRequest to FloatingLeaf forms by iterating through the returned map.

All these steps take place in `LeafRequestSource.to_floating_shapes()` which is called when a `FloatingCarriage` is (lazily built).

A `FloatingCarriage` is created on first compiling the shapes for a carriage. The lazy mechanism ensures that the layout algorithm is only called once and only if/when needed. It is through `FloatingCarriage` that the layout mechanism is ultimately invoked. `FloatingCarriage` also contains the mechanism to convert from `FloatingLeaf` to `AnchoredLeaf` via `unfloat_shapes()` and so is very much the front door of the allotment process.

## Layout tree nodes

Every node in the layout tree, whether container or leaf, ultimately implements `ContainerOrLeaf`. It does this through one of two implementations. `Container`, which is the class for all layout tree non-leaf nodes (container nodes) implements it, as does `FloatingLeaf`, the only valid layout tree leaf type. There are different kinds of container (see `allotment.md`) and each is implemented as a trait implementation of `ContainerSpecifics` which is passed into the container.

There are three non-trivial methods in `ContainerOrLeaf` (along with various trivial accessors, etc).

* `get_leaf` -- to build a tree, `get_leaf` is called repeatedly on the requested leafs. In the process of "getting" the leaf, all intermediate containers are created.

* `build` -- this is the first, "up" phase of the up-and-down algorithm, where the sizes of nodes are computed from their children. It is run from `full_build` in `Root`.

* `locate` -- this is the second, "down" phase of the up-and-down algorithm, where the locations of nodes are computed from their sizes.

`ContainerSpecifics` has two methods corresponding to the last of the two functions above:

* `build_reduce` is called during the build phase.
* `set_locate` is called during the locate phase.

## Implementing container and leaf nodes

During the up phase, `Container` first calls `build` on child nodes and when that is done passes the computed `ContentSize` objects to `build_reduce` which then needs to produce a single puzzle value of a float, its own height. In the process an implementer will typically calculate the relative positions of the elements it has been supplied and not only return the total height, but store away the offsets of each child for the second phase.

During the down phase, `Container` calls `set_locate` and leaves the implementation to call child objects itself. It is passed its own top and uses this and the relative offsets stashed away during the first phase to know what top values to pass to each child.

Tracing through containers can be confusing through extensive use of the puzzle system. If you have trouble, I recommend doing a deep dive into the puzzle methods used. `stacker.rs` is a good introductory container class, neither trivial (like `overlay.rs`) nor complex (like `bumper.rs`). The bumping code is surprisingly similar to the other nodes as most of thework is done by code elsewhere (see `collision.md`) to keep all this layout tree complexity out of that algorithm. The pattern is the same as for other nodes. At the end of the `build_reduce` call, `Bumper` has all the information which it needs to bump and so does so. It then squirrels away the results and reports them during the `set_locate` phase.

Leaf nodes are of type `FloatingLeaf` (in `floating.rs`). (They implement `ContainerOrLeaf` as well, of course, which is how they interact with the layout tree). Already, when the shape was first added to the `LeafRequest`, the space it occupies has been added to the `LeafRequestSize` object inside `LeafRequest`. On the call to the `build()` method of `ContainerOrLeaf`, the `FloatingLeaf` implementation extracts this and returns it as the seed of the very start of the up-and-down algorithm. The bounds are transferred from the `LeafRequestSize` to `ContentSize` which are very similar objects but differ in that `ContentSize` is aware of the "exchange-rate" of pixels for base-pairs, which `LeafRequestSize` cannot be (the same `LeafRequests` being applied to various scales). 

For various, annoying and tirival reasons, `Root` cannot be of type `Container` without messing up the code while sharing many of their properties (having child nodes etc). For this reason `HasKids` exists which absstracts away much of the management of child nodes and can be used both by `Container` and `Root`, avoiding code duplication.

## Reporting Metadata

A somewhat annoying ripple in the abstraction, but a necessary one, is the handling of reporting metadata; that is those shapes which rather than appearing on the screen generate values for track reports. These are returned along with size data in `ContentSize` during the up-phase of the up-and-down algorithm, and are recorded in the carriage state at the root. In this way, they follow the data path of the playingfield data for overall image size.
