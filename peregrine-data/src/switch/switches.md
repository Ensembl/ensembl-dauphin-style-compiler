switches are at the heart of peregrine. They are a form of registry with actions bound to them. Switches control almost all actions of the genome browser.

Switches map values onto a tree with string indexed arcs. For example:

```
               [root]
               /    \
            track  settings
            /         |
          gene      colour
          /   \       |
       labels style   v
          |     |   "blue"
          v     v
         true { "fancy": true, "x": "y" }
```

Note that while switches can have fancy values -- essenitally arbitrary JSON objects -- the switches mechanism doesn't care about these values except for whether or not they are "truthy". In this context `null`, `false`, `""`, `0`, `[]` and `{}` are falsy; all other values are truthy. These were chosen so that there is exactly one falsy value for each "outer" type of data.

You specify switches by string indexed path. Intermediate nodes are created, as needed, with value `null`, which is falsy.

Also associated with each node in the switch tree can be zero-or-more Tracks, some of which may be (boolean) flagged as "triggers".

A track contains information about a certain style-compiler program which can be run in certain circumstances to generate information on the screen.

The output of the switch tree is a TrackConfigList. This is a list of Tracks which should be run. The current TrackConfig list is determined *only* by the current switches tree and no other factors. It is lazily regenerated whenever requested after the switch tree is altered.

The algorithm to generate the TrackConfigList is to recurse down the switches tree from the root through all truthy nodes, adding any tracks with the "trigger" flag set. Note that child nodes are only scanned for truthy values, so truthy descendents can be "blocked" by a falsy parent. This allows configuration of things currently turned off when they are eventually turned on without exposing that config prematurely.

When a Track is added to the TrackConfig list, a TrackConfig is generated for it. This is stored in the TrackConfigList and can be retrieved by an accessor given the track. A TrackConfig is a subset of the switches tree. For each track the switches tree is recursed
and any mentions of the track are added to the TrackConfig tree (even those flagged as non "triggers" this time). Note that, again, falsy nodes are not searched to further depth though this time the falsy node itself appears. The result is the subset of the switches tree "of interest" to a given track.

A TrainTrackConfigList is derived from the TrackConfigList and takes into account other factors, such as current scale, to further filter the tracks to be run.

So overall the flow looks like this:

```

Switches ----> Track-    --(subset)--> TrainTrack-
tree           ConfigList      ^       ConfigList
 ^    ^                        |
switch operations          scale etc
```

