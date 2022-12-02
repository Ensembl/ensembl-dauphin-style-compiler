# Leafs

## Introduction

Leafs are the genome browser's name for what in most layout systems would be called layout boxes. They are caleld leafs for two reasons: the word box is annoyingly reserved in lots of contexts in Rust; and in the genome browser you can only draw shapes at the bottom of the box tree, at its leaves. There is no notion of mixed contennt: a node is either a leaf or a container, and containers are of only marginal importance, so everything of significance is with reference to a leaf.

In the genome browser, co-ordinates are stored in SpaceBases and SpaceBaseAraas. These include three continuous co-ordinates and a fourth polymorphic quantity, the allotment. The allotment contains any "other" geometric information required to specify a coordinate (such as the containing box). At various times more or less such information is required depending on the stage through the pipeline which the Spacebase is used. Initially these co-ordinates float with respect to some leaf (the exact position of which is uncertain) and so the allotment is simply a leaf or even a mere leaf name. Ultimately further down the pipeline, the leaf will be placed and so the continuous axes of the spacebase have offests applied, and so on, so less and less information is stored in the allotment. However, for much of the code the allotment includes some kind of leaf.

## The Leaf Pipeline

When a shape is created it is with reference to the path to some leaf. As these creation steps can occur in any old order and intermixed with style requests for these leaves, there is little can be done until the program is over than store these requests by the given path in an only very lightly-parsed form. This is called a `LeafRequest`.

Here is a cartoon of a shape at the LeafRequest stage. Note that the leaf part (in the inner box) is just a path.

```
+---------------------+
| Rectangle           |
+---------------------+
| start: ...          |
| end: ...            |
| colour: ...         |
| leaf:               |
|  +----------------+ |
|  | track/123/xyz  |<+--- the embedded LeafRequest
|  +----------------+ |
+---------------------+
```

Jumping to the other end of the pipeline for a moment, ultimately, these leafs will map to a very few parameters, most importantly screen-offset and height (having somehow accounted for all other boxes), and a few auxilliary things like z-index. This is called the `AnchoredLeaf` stage.

```
+---------------------+
| Rectangle           |
+---------------------+
| start: ...          |
| end: ...            |
| colour: ...         |
| leaf:               |
|  +----------------+ |
|  | offset: 125    |<+--- the embedded AnchoredLeaf
|  | height: 32     | |
|  | depth: 42      | |
|  +----------------+ |
+---------------------+
```

Finally these offsets will affect the co-ordinates of the shape in a shape-specific way, offsetting values. The only things remaining in the leaf will be those few auxialliary values. This is called the `LeafStyle` stage.

```
+---------------------+
| Rectangle           |
+---------------------+
| start: ... +125     |
| end: ...   +125     |
| colour: ...         |
| leaf:               |
|  +----------------+ |
|  | depth: 42      |<+--- the embedded LeafStyle
|  +----------------+ |
+---------------------+
```

In all these stages, shapes with their leaves are just unordered collections of things with no relation to each other. But what of the leap between amere string specifier and the AnchoredLeaf? How are the boxes laid out? How can independently-computed panels bump andoffset in a coherent way without tearing? This is the purpose of this document.

```
LeafRequest ---> ??? ---> AnchoredLeaf ---> LeafStyle
```

Don't confuse the style tree and the layout tree!

## Building the layout tree

There are a number of stages to going from a mere LeafRequest to apositioned AnchoredLeaf. They are:

1. Collate together all the style-declarations and their patterns for a given program into some data-structure which can be queried easily;

2. Build a tree out of all the LeafRequests, with each node representing a position on a path, out of object nodes with algorithmic intelligence (bumping, etc), using the styles computed in the previous step to choosethe object type and initially configure it.

3. Apply a recursive algorithm through the tree (named "the up and down algorithm") to assign offsets and heights to every element in the tree, bumping, stacking, etc, as implemented in each of the objects in the previous step. The result being a mapping from a path to a height, offset etc. Note there is an entry *per unique path*, if many shapes use the same path, there will be just one entry.

4. Map every shape from its LeafRequest, through the map prepared in the previous step to its FloatingLeaf.

Note that the position of an object *cannot* be completely determined by the above algorithm because of interactions with other panels. This is solved with the *puzzle* system. That will be described separtely later.

Here are those four steps diagramtically:


Step One: Build a tree for the patterns. 
```
                    (root)
                   /   | \
                  /    |  \
                  v    v   v
                  a    *   b (padding=2)
                 / \       |
                 v  v      v
      (magrin=1) c  d      ** (depth=23)
```
The style declarations in the program will assign properties to the nodes in their path (shown here in brackets).

Step Two: Build a tree for all the LeafRequests, attaching any styles from step one. On the left is the complete set of leafs reference in all the shapes. They are built into a tree. Again, hypothetical properties are shown in parentheses.

```
[a/b/c/d]                (root)
[a/b/x]     ->           /    \
[m/n/p]                  a    m (priority=6)
                         |    |
               (depth=6) b    n 
                        / \
                        x  c
                           |
                           d (coord-system=window)
```

Note that the nodes here now represent significant objects which can bump and otherwise lay out their children according to their kind.

Step Three: apply "the up and down algorithm" on the tree to assign offsets and heights for each node. Note in parentheses that every node now has two numbers associated with it, offset and height. (Other, pre-existing properties are not shown, but will still be present).

```
           (root)
           /    \
     (0,0) a    m (12,2)
           |    |
    (0,10) b    n (14,2)
          / \
    (0,5) x  c (5,2)
             |
             d (7,3)
```

Step Four: the tree is used to create `AnchoredLeaf`s for each shape:

```
[rectangle ... a/b/c] -> [rectangle .... (5,2)]
[circle ...... a/b/c] -> [circle ....... (5,2)]
[rectangle ....m/n]   -> [rectanggle .. (12,2)]
...
```

So we can fill in our picture

```
LeafRequest ---> FloatingLeaf ---> AnchoredLeaf ---> LeafStyle
```

Note that FloatingLeaf and AnchoredLeaf actually contain the ultimate LeafStyle inside of them! It is generated during this process. It is simply that once a leaf has been anchored and the transforms applied, the rest of their contents is sloughed, leaving the LeafStyle visible.

## The Up And Down Algorithm

The Up and Down algorithm is the mechanism by which offsets and heights are assigned in Step Three and is the core of the box algorithm. It is the general framework for interaction between nodes but which leaves nodes still with the flexibility to implement complex transformations between their children.

The first stage, "Up" starts at the leaves of the tree and has them report their intrinsic height (and width for bumping) to their parents. The data flows *up* the tree. When it reaches the top of the tree, the root node knows the size of its contents. More importantly, every node should take note of the reported height of their child nodes (and if necessary do layout computation on them).

In the second stage, "Down", the tree is travesrsed downwards, passing down the top offset (initially zero) to child nodes. The intermediate nodes can then alter this offset if they wish as they pass the offset down to their children.

By the end of Up and Down, every node has both a height and an offset, heights having gone up and then offsets down.

For example, a node which bumps its children will listenfor their height and width in the up phase and, either at the end of its up call or start of its down, arrange the chilrden's relative offsets appropriately to avoid collisions. When the bumping box learns of its own top offset it can the add the two together to tell each of its children their absolute offset.

The puzzle system, alluded to earlier and described later, adds further flexibility to this algorithm.

## The Puzzle System Itself

The puzzle system is a way of delaying computation of a value containing some unknown quantity until the last moment. It effectively allows functional programming in the sense that a function can declare ("I don't know what my value of X is, but it's 6 more than theirs" or such like). In the end a kind of graph of interacting unknowns is created and when the value is finally known then all the other values resolve.

Cartoon of the puzzle system
```
    +----+                         +----+
    |????| <- top of my box        |????| <- top of sibling box
    +----+                         +----+
     |   |                              |
     +3  +--------------------------|max|
     |                                |
     v                                V
    +----+                          +----+
    |    | <- top of my child box   |    | <- height of my box
    +----+                          +----+
```

The puzzle system is completely generic: it doesn't have any notion of heights, offsets, and so on, it's a pure computation system which accepts most types and closures. These maps of values are called puzzles.

Eventually you will want to plug in some numbers to get anwsers. Puzzle supports multiple such solutions being calculated simultaneously. Each solution iscalled an Answer and an Answer's scope is across a whole puzzle, not just one value. You can set an unknown to a value *in the context of one Answer* and to some other value *in the sontext of a second answer* and the derived values will also likely differ when they are retrieved *in the context of the relevant answer*.

```
 One puzzle:                     1. make Answer1; set A=23 for Answer1;
 +------+       +------+         2. make Answer2; set A=42 for Answer2;
 |??????| -+2-> |output|         3. read B=25 for Answer1;
 +------+       +------+         4. read B=44 for Answer2.
     A             B
```

The puzzle algorithm takes great pains to make sure constant values and expressions (even sub-expressions) are only calculated once where there is no dependence on unknowns.

## Use of The Puzzle System

The Up and Down algorithm doesn't actually report native values for height and offset, but puzzle value versions of them. These can,of course be constants, or else they can depend on other terms, even complex caluclations. It is in this form that the FloatingLeaf form of shapes are stored, and this is the most important form of shapes, it is how they are cached and composed in the code.

Every "Train" (ie single display scale for a given chromosome and track setting) has its own Answer. The conversion from FloatingLeaf to AnchoredLeaf is essentially simply solving the puzzle for a given train's answer andrecording the results in each AnchoredLeaf,

One use of this is in bumping. A bumping node needs to consult with neighbouring nodes as to where items are placed, and needs the combined bumping output across all panes before it canknow its track height. Even normal tracks can be made artificially higher by their neighbours.

This is made possible with puzzles. The up and down algorithm is run for each panel, leaving puzzle nodes including references to the global bumping algorithm. Because all nodes always get puzzle values, children have no way of knowing whether the values they are receiving come from the bumping algorithm or are constants or any other source, and shouldn't need to care). So nodes arbitrarily compose without complexity as the combinations grow.

This is why the puzzle algorithm goes to great pains to optimise constant expressions: for most boxes and their children the values *will* be constants and the up-and-down algortihm *will* yield a normal constant value. This is particularly the case for boxes "deep" in the tree, dealing with small regions, which are necessarily the most numerous and so the most important for performance. If a handful of outer boxes are more complex, we can afford the extra time. Using puzzles means that we don't have to take the usual approach of hard-and-fast layering to allow such differences to be implemented.

One example of the efforts puzzle goes to for constant pre-evaluation is the `commute` function which takes a list of puzzle values and a callback to combine them to create a single new value (such as adding them all up). This is a bit like reduce in map/reduce. However in this case (because you'vecalled commute and not another compose function) puzzle rearranges the values and applies the operation to all the constants at creation time, leaving only any variable parts (if any) to add later when they are known. For example, if there are eight values and two unknowns, those eight will be (say) added at creation time whatever order they appear in, leaving only two to add. Naturally puzzle includes memoization as well.
