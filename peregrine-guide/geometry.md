# Note

The functions described in this document are defined inside arrayutils but are generally accessed through the corresponding GLAxis call.

# Anchors

Co-ordinates in peregrine are complex, as relative sizes and positions can vary depending on scale, and these routine transformations are delegated to the GPU. We're therefore always dealing with all kinds of deltas, scales, and axis directions rather than simple co-ordinates. This can get incredibly confusing.

The metaphor we use to simplify modelling is ships on an ocean. Each object is a "ship". There are two, somehow ovelrying but independent, seas to which a ship can attach, the "paper" sea being the "document" and the "screen" sea, being the viewport. Ships use an anchor to attach themselves to one of these seas. As the seas move these locations stay constant but the ship is seen to move.

Ships are anchored with anchors. An anchor has two ends, a ship end and a sea end. The ship end, in general, can be any position within the ship relative to ship-board datums of left/top, centre, or right/bottom. The sea end is an offset from the left/top or right/bottom of either the paper or the screen sea. As an example, if we have an object whose centre we always wish to be at 1,200bp we describe it as follows: "The anchor has a ship end 0px from the centre of the ship and a sea end at 1200 on the paper sea". A further example might be a rectangle whose righthand side is 4 pixels to the left of the right of the screen. We describe this as follows: "The anchor has a ship end 4px to the right of the right of the ship and a sea end 0px from the right of the screen sea".

A ship can have one anchor point and a fixed size or else two anchor points and so "stretch" when scale changes. The latter are denoted by novel words derived from the former by incorporating the word stretch into the natural name of the shape. For example, a single anchor point rectangle (which might, for example, be an icon) is known as a "rectangle" whereas a rectangle which grows and shrinks with scale (for example an exon) is known as a "stretchtangle".

As ship and sea start with the same letter, the last is used [A] and [P] to denote each end.

The type of the sea end (in each axis) determines the applicable geometry, so by the time it reaches the point of implementing geometry, this will be known and invariant. Even when there are two anchors, both can only beof a single type so there are only four options. Axes operate independently and can be considered separetely.

An object can have one or two anchor points.

## One Anchor point

Shapes with a size can have a single anchor point.

```
<--------size-------->
+--------------------+
|                    |
|    [P]             |
|     |              |
+-----+--------------+
      |
     [A]
```

The ship anchor can be with reference to one of three datums (min,centre, max), being of the users choosing. A delta known as a "walk" ("..." in the diagrams) is applied from this datum to the ship anchor point. This walk is always positive to the right/down from the datum to the ship end of the anchor.

```
L:                        C:                        R:
<--------size-------->    <--------size-------->    <--------size-------->
[D]-------[D]------[D]    [D]-------[D]------[D]    [D]-------[D]------[D]
|......              |    |     ......         |    |     ...............|
|    [P]             |    |    [P]             |    |    [P]             |
|     |              |    |     |              |    |     |              |
+-----+--------------+    +-----+--------------+    +-----+--------------+
      |                         |                         |
     [A]                       [A]                       [A]
```

The first example above has a positive walk, last two examples have a negative walk.

The goal is to calculate start and end distances (s1) and (s2) being the left and right walls of the object, either from the left/top or right/bottom, depending on how the sea end anchors are arranged.

```
- (left/top):
                    <--------size--------> 
                    [D]-------[D]------[D]
(s1)===============>|                    |
                    |......              |
                    |    [P]             |
                    |     |              |
(s2)================+=====+=============>|
                    +-----+--------------+
                          |               
                         [A]              

+ (right/bottom):
                    <--------size--------> 
                    [D]-------[D]------[D]
                    |                    |<============(s2)
                    |......              |
                    |    [P]             |
                    |     |              |
                    |<====+==============+=============(s1)
                    +-----+--------------+
                          |               
                         [A]              
```

Initially considering the left/top (-) forms, in the case of min (L), s1 is the left wall and the anchor and walk are subtracted, and s2, the right wall, lies at [size] distance away.

```
-L:
  (s1) = [A] - [walk]
  (s2) = [A] - [walk] + [size]
```

In the case of max (R) the right wall is the simple subtraction, and the left wall is size away.

```
-R:
  (s1) = [A] - [walk] - [size]
  (s2) = [A] - [walk]
```

In the case of centre (C), the result is symmetrical. The centre is at [A] - [walk] so the edges are at

```
-C:
  (s1) = [A] - [walk] - [size]/2
  (s2) = [A] - [walk] + [size]/2
```

When we have anchors from the end rather than the start, the sign of the walk is flipped (as walks are always left-to-right). It is simplest to think of the size transformation as similar, that shapes have a "negative size", which naturally flips the roles of s1 and s2, as required.

```
+L:
  (s1) = [A] + [walk]
  (s2) = [A] + [walk] - [size]

+R:
  (s1) = [A] + [walk] + [size]
  (s2) = [A] + [walk]

+C:
  (s1) = [A] + [walk] + [size]/2
  (s2) = [A] + [walk] - [size]/2

All the calculations have the form:
  (s) = A + fp * [walk] + fs * [size]
where:

    fp   fs1  fs2
-L  -1   0    +1 
-C  -1   -1/2 +1/2
-R  -1   -1   0
+L  +1   0    -1
+C  +1   +1/2 -1/2
+R  +1   +1   0
```
By inspection, fs2 = fs1-fp.

This is provided by the function calculate_vertex.

In some cases geometries supply anchor locations via a different data array to allow efficient scaling. In this case calculate_vertex_delta is used, which sets [A] to 0, a count is taken instead of anchor data.

## Two anchor points

An object may instead be defined to have two anchor points. In this case, the size of the object is defined by the relative co-ordinates of those anchors. The two anchors are known as systems one and two, respectively.

```
system 1      system 2
[D]----------------[D]
|......     .........| walk1 = 6; walk2 = -8
|    [P]   [P]       |
|     |     |        |
+-----+-----+--------+
      |     |
     [A]   [A]
```

Again, we ultimately need wall position co-ordinates.

```
            system 1      system 2            system 1      system 2
            [D]----------------[D]            [D]----------------[D]
            |......     .........|            |......     .........|
            |    [P]   [P]       |            |    [P]   [P]       |
(s1)=======>|     |     |        |            |<====+=====+========+======(s1)
(s2)========+=====+=====+=======>|            |     |     |        |<=====(s2)
            +-----+-----+--------+            +-----+-----+--------+
                  |     |                           |     |
                 [A]   [A]                         [A]   [A]
```

We see, therefore, that for the two-anchor object this degenerates into two independent problems.

```
-:
  (s1) = [A1] - [walk1]
  (s2) = [A2] - [walk2]

+:
  (s1) = [A1] + [walk1]
  (s2) = [A2] + [walk2]
```

So our two anchor problem becomes a degenerate case of the one-anchor problem for the case when size is zero. In the case of independent sea-end anchors, we also have the degeneracy that [A] is zero, so the calculcation degenerates into an optional negation of the walk coordinates, depending on sea axis direction.
