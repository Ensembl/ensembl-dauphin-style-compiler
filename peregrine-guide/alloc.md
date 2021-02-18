# Packing

## Approach

We use a shelf-based algoithm (rather than, say guillotine or skyline) for speed. We use best-fit into open shelvesand presort by height. There is always sufficient vertical space on a shelf because of strictly decreasing heights.

As an optimisation, if a significant underpart of the shelf is lost, the shelf is split and stored in a pending array until the height is sufficient to handle new requests. We can't add the subshelf immediately as it may be smaller than incoming objects and, as mentioned above, we always rely on there being sufficient height.

```
+===========================+==============+====+========+===+===========================+
|                           |              |    |        |   |                           |
|                           |              |    +--------+---+                           |  < - a shelf
|                           |              +----+                                        |
|                           +--------------+                                             |
+---------------------------+                                                            |
+========================================================================================+
```

Splitting a shelf:

```
+===========================+==============+====+========+===+===========================+
|                           |              |    |        |   |                           | <- shrunken old shelf
|                           |              |    +=====+==+===+===========================+
|                           |              +----|           |                            |
|                           +--------------+    +-----------+                            | <- new sub-shelf
+---------------------------+                   |                                        |
+========================================================================================+
                                                ^
                                        height triggered so shelf split
```