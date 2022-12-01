# Leafs

## Introduction

Leafs are the genome browser's name for what in most layout systems would be called layout boxes. They are caleld leafs for two reasons: the word box is annoyingly reserved in lots of contexts in Rust, and in the genome browser you can only draw shapes at the bottom of the box tree, at its leaves. There is no notion of mixed contennt: a node is either a leaf or a container, and containers are of only marginal importance so everything is with reference ot a leaf.

In the genome browser, co-ordinates are stored in SpaceBases and SpaceBaseAraas. These include three continuous co-ordinates and a fourth polymorphic quantity, the allotment. The allotment contains any "other" geometric information required to specify a coordinate (such as the containing box). At various times more or less such information is required depending on the stage through the pipeline which the Spacebase is used. Initially these co-ordinates float with respect to some leaf (the exact position of which is uncertain) and so the allotment is simply a leaf or even a mere leaf name. Ultimately further down the pipeline, the leaf will be placed and so the continuous axes of the spacebase have offests applied, and so on, so less and less information is stored in the allotment. However, for much of the code the allotment includes some kind of leaf.

## Creating Shapes

Initially shapes are created with a `LeafRequest` allotment. This is little more than an allotment name and a placeholder for accumulating further information. Here they sit in `UnplacedShape` objects inside `AbstractCarriageBuilder` in caches and so on until they are called upon for composition. `LeafRequests` are created by `LeafLists` which takes care of deduplication (for effieicency). `LeafLists` can be merged. 



## Mapping shapes

When first requested, either by `spec()` or `get_drawing_shapes` the abstract carriage code pushes the state to the next stage through "building". Building is the process of positioning boxes. This occurs in two stages. First position_boxes is called on the `LeafList` to run the box positioning algorithm. This returns a `LeafTransfomrableMap`. This creates an `AbstractShape` for every leaf in the leaf list. An `AbstractShape` is a leaf with the property that as part of a given composition it can yield its corresponding `AnchoredLeaf`. The "given composition" is supplied as a `StaticAnswer` model from puzzle. In turn, the `AnchoredLeaf` contains methods for manipulating SpaceBase and SpaceBaseArea objects in the given shape type to the position of the box. This process is overseen by the `make_drawing_shapes` method of `AbstractCarriage` which is in turn invoked by `DrawingCarriage`. The `StaticAnswer` ultimately comes from the `Train` object. Once transformed the shapes have only a `LeafStyle` as their allotment, remaining vestigial information about z-index and so on. Everything affecting the primary co-ordinates has been shifted into the continuous axes.
