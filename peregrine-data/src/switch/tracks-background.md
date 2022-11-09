# Tracks in the Genome Browser

This document provides the background orientation for track handling in the genome browser. Although it's only background, it is probably essential to read before diving into the technical documents.

## The visual meaning of track is ambiguous!

When looking at a genome browser it's very tempting to come up with a *linear* arrangement of tracks. That is, that there is a single level of entity called a track which appears in some order, one after the other vertically.

Experience with the old ensembl genome browser suggests that this doesn't scale well. The usual approach is to come up with various non-first-order concepts like "track groups", "sub tracks", "parent tracks", "container tracks" and so on. In practice, the interaction of these things causes confusion and chaos in the code, and means a lot of things are not possible. This process has recurred across different projects in different teams and organisations.

But what you do pretty consistently get, at least visually, is a hierarchical organisation. Though it's not a strict visual set of boxes (there are often miscellaneous things which overlay, underlay or join multiple boxes), it's usually possible to come up with the intended hierarchy by looking at the visuals.

These two paragraphs might look contradictory but there is an important distinction. There is no hierarchy of levels in the latter: no "level" in the tree which is at the track level, or the subtrack level or parent track level: no named levels with particular properties. This is a similar distinction between traditional, Linean classification with "kingdom", "family", "genus", "species", and so on, compared to cladistic methods where the levels are unmarked. There is no identified *special* level meaning "track" or anything else.

Nice and flexible as this vague and underspecified representation is, practically we need more to do anything meaningful with this data. In particular we need

1. to store shapes in the leaf nodes
2. to flag how the shapes are combined in the non-leaf nodes (eg overlap, stack, bump, etc).
3. a way of identifying and refering to nodes

In effect, the visual model of tracks is a CSS-like box model. To visually "add a top level track" that means "add this data into the top level tracks box" (which will have some distinguished id). To add it as a subtrack, the data is added to a different box, lower down the tree.

```
                               [root-node]
                    +----------+   |     +-----+
                [#tracks]        [#ruler]     etc
              +---+   | etc
[#regulation-123]   [#variation-456]
   |       |
[#sub1] [#sub2]
```

Importantly, such trees can be *merged*. Different processes for retrieving and manipulating data can create multiple trees and then merged into a single tree. Typically, non-interacting tracks would use their own IDs to avoid collisions, like so:

```
[root-node]        [root-node]                [root-node]
    |                   |                          |
[#tracks]      +    [#tracks]      ----->      [#tracks]
    |                |      |            +------+  |   +-----+
[#track1]      [#track2] [#track3]   [#track1] [#track2] [#track3]
```

But it could be that different processes write to distinct subparts of some singe track.

```
[root-node]        [root-node]                [root-node]
    |                   |                          |
[#tracks]      +    [#tracks]      ----->      [#tracks]
    |                   |                          |
[#track1]           [#track1]                  [#track1]
    |                |      |            +------+  |   +-----+
 [#sub1]         [#sub2] [#sub3]         [#sub1] [#sub2] [#sub3]    
```

To manage this merging process, as well as nodes knowing what to do visually with their multiple decendents, nodes are given a priority which allows their parents to order them appropriately after merging.

So what we are looking at doing is a certain number of processes which generate separate box trees which are merged for the final display. Exactly how many processes there should be is discussed later, but you can imagine it relates to the data source, the design, etc.

## The configuration meaning of track is ambiguous!

So, now that we have a flexible notion of a track which doesn't require complex notions of track hierarchies, we're in trouble in terms of how the rest of the UI interacts with the genome browser, in terms of turning tracks on and off. Certainly the rest of the UI and APIs will have various ideas of what a track is (probably at least partially inconsistent with each other) and we don't want to be forced to reintroduce this "special level" of "The Track" by the backdoor through this API.

For exmaple, the UI probably *does* have a linear idea of "track" and various concepts of sub and supertracks, just so that a user can configure things. This will presumably evolve over time, necessarily creating new code for configuration in the UI, etc. Also, a single track may be configurable in exciting ways (such as only showing particular types of variant). But we don't want such complexity to pollute the genome browser by the back door!

Instead, we have a second tree rather like the box tree, again without distinguished levels of any particular, fixed name or meaning. Indeed, in many places there may well be a one-to-one coreespondence between the two. This tree, the switch tree, is the place settings are stored. Each node of the tree has some setting value which can be exampined by the processes generating the data. Certain nodes can be "special" in that when they are set to true (in some sense), they cause a process to be added to the list of processes to run.

By this means, which is essentially the same as the previous means, even though the data-structures are distinct, again we sidestep the "what is a track" issue by leaving it to convention. Note that while many values will be boolen (ON or OFF in the diagrams below), in theory there is any value at a node.

```
                               [root-node]
                    +----------+   |
                [#tracks]         etc
              +---+   | etc
[#gene-123:ON]       [#variation-456:OFF]
   |       |                   |
[#sub1:ON] [#max-genes:47]    [#consequence:SERIOUS]
```

Note that it is only by convention that a certain level is the "track" level, to the extent that makes sense to the UI. As you go down the tree you slowly slide, at each branch, from large groupings, into individual tracks, then to subtracks, and then to their settings, the extent to which each layer exists being determined by what's meaningful for its parents.

So now we have an outline of the full picture:

> Certain settings in the switch tree cause processes to run. Each process generates a box tree. These box trees are then merged for the screen.

## What causes a process to run?

We've already mentioned that processes are associated with one or more settings to cause them to run. However, they must pass two other minor tests, which can exclude them.

1. It must make sense for this chromosome.
2. It must make sense at this scale.

The scale one is subtle. In effect, in the design, very different data is sometimes shown at different scales (eg the contig track becoming sequence). Even if the data is the same, it may be very visually distinct, to the extent it makes sense to consider them different processes. So the processes to run is scale-dependent. This is fine because as long as they generate a box-model with the same IDs and as long as only one is configured to run at any given scale, they work as if "a single track". The idea that a single track might run one of several independent processes depending on scale is a wrinkle in the naive idea that each process somehow corresponds to a track.

Chromosome filtering is simpler. Each track includes a predicate which has as its atoms the stick-id or a "flag", being a set of strings which is associated with each chromosome and might be things like "#has-variant-data" or "#no-regulation" or whatever.

## So, what is a process after all this?

A process in the above discussion is a style script. A style script's task is:

1. to retrieve some data, from somewhere;
2. to process it somehow if necessary;
3. to turn it into a bunch of shapes (in the visual tree described near the start of this document).

To do this it can rely upon:
1. functions to alllow it to make backend-requests;
2. various settings it gets when it runs;
3. a set of functions to manipulate data;
4. functions to create boxes and shapes and put shapes into boxes.

When considering when to split into multiple scripts, consider whether:
1. the actual kind of data wanted has changed significantly;
2. the means of retrieving that data has changed significantly;
3. the processing done to the data is significantly different;
4. the shapes to be generated is significantly different (different design).

Through all this there are two "stages of separation":
1. it could be a completely different script;
2. it could be the same script with some changed settings.

## Advice on structuring the switch and shape trees

It's easy enough to say as above "it's all flexible", but what should be done? Overarching the detailed advice are the following two recommendations.

1. Expect any decision to change over time, and don't over-engineer for future use cases too early. The system has been designed to make this simple.

2. Just because flexibility exists in the system, there's no obligation to use it.

That said, we can consider in turn the two tree structures.

### The shape tree

In terms of the shape tree, we should expect there to be some top-levels dealing with unfortunately necessary nuiscances of rulers, side-panels, and so on. These top-levels can largely be ignored.

Within this trivia there will probably be a single element with a known name called something like `tracks` which is where the important stuff goes. Initially, this can probably be ordered linearly, where each child is the thin stripe of screen which we tend to call a visual track. Within these elements it is up to the individual track what structure it wants to use.

```
                      [root]
                  +----+ | +----+
                  |      |      |
            [#tracks]   ...trivia...
  +---------+---+-----+      
  |         |         |
[#track1][#track2][#track3]
  |  | |   |    |   |  |  |
... per-track, we don't care...
```

Each track should be named (placeholders #track1, #track2 etc in the above) according to a setting passed into the program which somehow corresponds to the track's id in the UI. (At the moment it's hardwired, but that's bad). The order should also be passed in as a setting (ditto).

When additional complexity comes along, with supertracks and subtracks, after appropriate modelling layers can be inserted above and below (some) individual tracks.

The switch tree structure would be surprisingly similar in the trivial case. At the top level would go things like any future global settings (night-mode, special font, colour-blind mode, whatever) and probably a single, distinguished node for tracks in the regular, track-api sense, which may as well be called "#tracks" again. There may (or may not) be other nodes at this level for user tracks, BLAST tracks, etc. Note that for reasons not discussed here to do with lazy-loading, where a node is expected to have a massive number of children such that preloading feasable, it makes sense to do this at a separate node in the switch tree. So it makes sense for track-api tracks to be at a different place to user tracks to various trivia.

Beneath this level, again it is up to the individual tracks what to put there.

```
                      [root]
                  +----+ | +------------------+---------------+
                  |      |                    |               |
[#tracks (track api)]  [#user-tracks] [#special-tracks]   ...trivia...
                |              |              |      |
  +---------+---+-----+        |              |      |
  |         |         |    [#bigbed-1]   [#focus] [#ruler] etc
[#track1][#track2][#track3]
  |  | |   |    |   |  |  |
... per-track, we don't care...
```

# Further reading

For information on how scripts bind to the switch tree and get their settings see `track-payload.md`.
