# Collision

## Overview

The main interface to the bumping algorithm is `bumpprocess.rs` which contains the type `BumpPersistent`. The "persistent" in this type suggests (correctly) that the object should be kept around between the laying out of different carriages (in fact it is kept in the state for a train). THough a new `BumpPersistent` each time would be able to bump satisfactorily, by using the same persistent object each time, the algorithm can attempt not to jump things around when panning through a particular scale, by taking into account the prior positions of objects.

The key method in a `BumpPersistent` is make which takes `BumpRequestSet` and bumps its contents, returning a `BumpResponses`. This is the method called by the `Bumping` container object (in `containers` for doing the bumping. 

`BumpRequestSet` lives in `bumprequest.rs`. It is the means by which the boxes to bump are specified. A `BumpRequestSet` contains a set of `BumpRequest`s which are individual boxes requesting to be bumped. The reason for the extra container type is that each container is identifiable to the algorithm and so it can take into account what it has and hasn't seen before to avoid repeating work, and to avoid jumping, at the set level.

A `BumpPersistent` maintains an `Algorithm` object. Items can be added to an algorithm object and the new items will be placed consistently with anything already in the object from previous runs, if possible. If no such incremental addition is possible, the Algorithm object will fail to bump and BumpPersistent will create a new one, adding all the items to be bumped -- old and new. When this is forced to happen, old object may unfortunately jump to new positions. So there are three possible results from the `make()` method:

1. this is exactly the same set of `BumpRequestSets` as before, and so nothing needs to be done;

2. the `Algorithm` object accepts any new `BumpRequestSets` without failing, and so the old algorithm is retained, the old objects staying in position and the new ones placed in some free space.

3. the `Algorithm` object rejects the new `BumpRequestSets`, and so it is destroyed and a new one created, with objects old andnew getting new postions.

An `Algorithm` is first created with an `AlgorithmBuilder`. This accepts an arbitrary number of `BumpRequestSet`s and will never fail. On building, an `Alogirthm` is guranteed to be successfully returned. By contrast, extra objects are added to an `Algorithm` object directly, through the `add` method, which can fail due to "pre existing commitments" for spacing other sets.

## Removing the annoyance of "infinite" objects

There are two types of object which may be bumped: (most commply) objects of finite horizontal extent, which occupy some region of the genome between base-pairs A and B; and (rarely) objects of "infinite" extent, which would stretch across the whole window no matter how zoomed-out we were. The latter are not super-useful but are permitted by the model which the collision algorithm must use, and could potentially be UI elements within the browser.

"infinite" objects are always placed at lower offset than all finite objects and are called, in places, the substrate. When these are encountered, a simple accumulator adds their total height and allocates their base. Once all the infinite objects have been accounted for (almost always none) the initial offset is recorded in the responses and added to the result of any finite objects, which are therefore bumped as if they start at position zero. In this way infinite objects, an annoyance, are rapidly disposed of.

```
+--------------------------------------------------------+ -                 zero offset
|   infinite object 1                                    | |                      |
+--------------------------------------------------------+ | substrate            v
|   infinite object 2                                    | |                greater offset
|                                                        | |
+-----------------+--------------------+---------------+-+ - <- "zero" for main bumping algorithm.
|   finite object |                    | finite object |        substrate offset added to queries
+--------+--------+------+             +---------------+        "on the way out of the door".
         | finite object |
         +---------------+
```

## Skylines

Fnite objects use `Skyline` a datastructure from the toolkit code. A Skyline is the core data-structure of bumping. It maintains a piecewise-continuous maximum value along a discrete dimension (i64). Initially this maximum is zero everywhere but pieces can be added to it. A piece comprises a range along the dimension, and a height. The height of the waterline is set so that the height in the range supplied is set to the maximum existing value in that range plus the height given. Skyline also keeps track of the maximum value used anywhere. Because of the nature of bumping, these maximums are usually best imagined being in the "down" direction. For exmaple skyline might do

```

  0       3        6        10   11
0---------+                       +-------
1         |                  +----+
2         +--------+         |                <-- BEFORE
3                  |         |
4                  +---------+

Add (5-8) height 2

  0       3     5      8    10   11
0---------+                       +-------
1         |                  +----+
2         +-----+            |                <-- AFTER
3               |            |
4               |      +-----+
5               |      |
6               +------+
```

Internally a range is known as a "node" and is stored at its leftmost position. For example in the above example, BEFORE has nodes at 0, 3, 6, 10, 11 and those nodes have height 0, 2, 4, 1, 0. Note that a node does not have an end, it remains in-force until superseded by another node.

## Bumping finite objects

When an `Algorithm` is first created, finites are bumped simply by taking the initial (all-zero) skyline and then adding the objects one-by-one. And we're done.

When attempting to add a new `BumpRequestSet` to the `Algorithm`, however, the procedure is considerably more complex.

## Rejected updates

We first hunt through the list for various circumstances which mean we should reject the whole update outright. 

We don't attempt to handle "bridging", that is adding a missing carriage between two existing carriages already known by the algorith, It's hard to know how this could ever happen but the correct response is clear: fail and let `BumpPersistent` rebuild.

```
Bridging (rejected for incremental adds):

      +--------------+              +--------------+
      | carriage 6   |              | carriage 8   |
      | in Algorithm |              | in Algorithm |
      +--------------+              +--------------+
                            ^
                            |
                      +------------+
                      | carriage 7 |
                      | to add     |
                      +------------+
```

After that the incremental algorithm looks through all the shapes to add and separates them into completely novel shapes and those which are laready in the algorithm (and are potentially being extended further to the left or right).

For existing shapes the Algorithm fails if:
1. they have changed their finite/infiniteness;
2. they have increased in height.

The latter should be almost as rare as the former (most boxes should remain the same height across carriages, as its' reccommended that at least space is reserved for an entire object whatever part of it is revserved). Increasing height would cause the algorithm needing to riple through tweaks to offsets which would make it more complex for this rare case. We just reject these borderline cases and let the bumping rebuild. 

```
BEFORE:
             :
             :
TERRA        +--------------+
INCOGNITA    | box known    |
             | height = 2   |
             +-------+------+-----------+
             :       | some other box   |
             :       +------------------+
             :

DURING:
"by the way that box is actually height 4, there's a bigger shape inside it, off to the left of the screen, which we didn't know about before".

+------------+--------------+
|more of same| box old known|
|box         | height = 2   |
|height=4    +-------+------+-----------+
|            |       | some other shape |
+------------+       +------------------+

EXPECTED: (but rejected as too complex: just rebuild from scratch)
+---------------------------+
| box adjusted height = 4   |
|                           |
|                           |
|                           |
+--------------------+------+-----------+
                     | some other box   |
                     +------------------+
```

This is why style programs should always reserve sufficient space for their object. Even if we supported this case, the "other boxes" would jump around as we became more knolwedgable. The only thing we are losing is some small subset of boxes not jumping in this case, for the saving of considerable complexity.

"New" infinite objects also immediately cause the algorithm to fail. Not only are infinite objects as rare as hens teeth, they should certainly always be specified on all carriages by the style program!

Note that in this section where we talk about the algorithm "failing" it only means that it is rebumped with a new Algorithm object and that is guaranteed to succeed. So we're just talking about causing a visual regrouping, not an error.

## Adding items to non-failed Algorithm objects

Now, certain that we are going to proceed, we can add all the objects. Again, what we do differs if an object is new to us or we have seen (part) of it before.

For an old object, we alter the skyline so that it is *at least as high as* the height of the old object for its extended range. We do this *before* adding the new objects. We can be certain that this place is free *if we add all old objects before all new*. A box extended off a current carriage will necessarily extend at least to the very edge of the current carriage (this is a requirement of style scripts). As mentioned above, we don't allow bridging updates (and this is why), so it can be considered to have "reserved" the space in any unknown carriages out to infinity unless we later find out otherwise. By setting th minimum height of the skyline out to its expanded range we call in this reservation if necessary. When this is done, new objects can be added.

A new object is simple, it is just added to the skyline like when we build the algorithm from scratch.

```
BEFORE:

"reserved"-----+
               v

 TERRA         A :+-------------------------+
 INCOGNITA     A :| box A                   |
               A :+---------------+---------+------+
                 :                | box B          |
               C :+---------------+--------------+-+
               C :| box C                        |
               C :+------------------------------+

before skyline:

------------------+                                +--------------
                  |                              +-+
                  |                              |
                  +------------------------------+

AFTER 1:
"box C extends outa bit more"

                 :+-------------------------+
                 :| box A                   |
                 :+---------------+---------+------+
                 :                | box B          |
        +------+ :+---------------+--------------+-+
        | box C| :| box C                        |
        +------+ :+------------------------------+

skyline:

--------+                                          +--------------
        |                                        +-+
        |                                        |
        +----------------------------------------+

AFTER 2:
"and there's a new box D"

                 :+-------------------------+
                 :| box A                   |
                 :+---------------+---------+------+
                 :                | box B          |
        +------+ :+---------------+--------------+-+
        | box C| :| box C                        |
 +------+-+----+ :+------------------------------+
 | box D  |
 +--------+

skyline:

-+                                                 +--------------
 |                                               +-+
 |                                               |
 |         +-------------------------------------+
 +---------+
```

Note that in this case a more sophisticated algorithm could have placed box D at the very top, as boxes A and B are actually complete. This would be at the expense of considerablecomplexity in the algorithm (managable in the case of a fixed bumping, but very messy when it comes to incremental updates and hard to make fast). (But have a go at doing it if you want!).

TO minimise the number of cases where overhangs "put a shadow under" other a considerable region of the screen, boxes in every add are sorted by length, the longer objects being placed first (and so closer to the bottom). It's therefor likely that The original circumstance in the example above would never has arisen: box B would almost certainly be closer to the bottom, meaning C at least one place furhter up, and so box D would also be at least one position further up. This is another good reason to reserve the full extent of a box in a style program even if there's no data in it: it helps the bumping algorithm place things in a sensible order!

Feel free to improve on this algorithm: I'm sick of it.

