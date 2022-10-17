# Switches

## Introduction

There are only three ways of altering what's on the screen with the genome browser.

  1. the stick (that is the genome and chromosome)
  2. the position within the stick
  3. switches

As you can see switches do much of the heavy lifting in configuring the genome browser. They broadly cover the are related to "tracks".

## Where Switches Fit

Switches form a uniform API and set of behaviours for managing tracks, however they don't do all the work for themselves. They are more like a "receptionist" for such information. To understand what switches do themselves, we can look at what's the next step down the line.

The next step are "style programs". A "style program" takes various settings and uses them (or hardwired values, as appropriate) to:
  1. retrieve data from some backend, 
  2. process it, and 
  3. generate some shapes for the screen.

The role of switches, sitting before this step, then, is to:
  1. choose which set of style programs to run,
  2. decide what their settings should be.

## The two faces of Switches

From the *outside* switches look like a tree data structure from which you can hang essentially arbitrary data. In this way they resemble a registry or, more loosely, a filesystem. It is this face of switches which the API changes to make things happen.

From the *inside* various nodes in this tree data structure have hooks called "triggers" attached to them which, when there is true data stored at that node causes a program to be scheduled to run. For example, there could be a trigger attached at "tracks/regulation-track" and associated with a particular program. If a user on the outside sets "tracks/regulation-track" to true, that program is added to the list of programs to run.

The program can also choose to receive other parts of the switch tree for further settings for its own, internal use.

## Where do the triggers come from?

Triggers come from the backend in payloads delivered at various points. They are configured from places like the track registry but will probably also typically include various hardwired entities, user-supplied tracks (via the javascript "backend"), and so on.

At the moment the mapping is relatively primitive, but I don't expect it to stay that way, either while I am here or after I have left! We certainly shouldn't even aim to make this 1-to-1 with the track api as that won't last!

## Why is it like this?

Switches are designed like this to decouple the external notion of a track from whcih style program to run. Externally "track" can mean at least three things:

1. a vertcial stripe on the screen reserved for some shapes;
2. some kind of configurable element in the UI;
3. some kind of data source from the data teams.

These three roughly correspond if you stand far enough away. However:

* Some data which is clearly a single integrated unit and deserves a single "campaign" of processing may be displayed in different vertical stripes (for example, the split gene track or, more pervasively, the idea of some data moving into a focus "track").

* Some data which is best handled separately may end up in tracks which may be superimposed or grouped together in some way.

* Some tracks will contain internal configuration which affects their behaviour. For example, we may want to turn on and off certain variant types, regulation tissues, and so on.

* Tracks could, in general, be influencedby various global settingswhich cross-cut tracks.

The external API for turning "tracks" on and off shouldn't care which style programs are run or how they are configured. For example, it shouldn't care that the "different gene tracks" are really just one program. Special kinds of tracks shouldn't mean remodelling the whole API to handle some new shape of data-structure. The external API for the application can come up with a mapping onto the switch tree which makes sense from the UI and data-production perspective, and leave the consequences to "behind" the facade of switches.

## Lazy-loading the switch tree

There may be nodes in the switch tree where it is not possible to know all triggers in advance because the branching is very broad or not determinable ahead-of-time. For example, we may not want to include the triggers for many hundreds of tracks in advance; the actual track could be a strange, run-time-generated entity; a user track may not even exist at boot time. To handle such cases, some nodes in the switch tree can be annotated with "expansions". When the API sets some value which "passes through" a node with an expansion, the relevant backend is notified and hasthe chance to specify a trigger in advance of the value being set, allowing it to be immediately triggered.

## What do these payloads look like?

Both boot-time and expansion-triggered trackconfig payloads are the same. They comprise zero-or-more of the following structures:

  * the name of a style program
  * one or more points in the switch tree which act as triggers
  * zero or more additional points in the switch tree to include as configuration
  * some settings to set to fixed values when you run the style program
  * some tags to force the program only to be run with particular sticks
  * some parameters to force the program only to be run at certain scales

(Some of these items will probably include an extra layer of indirection pretty soon to decouple parts of the system).

An expansion request payload is simply its name and the next branch point in the path whichthe user has chosen. The response looks like the above.

## Deep Dive

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

Switches ----> Track-    --(subset)--> TrainTrack- -----> Style Programs
tree           ConfigList      ^       ConfigList
 ^    ^        (lazily made)   |
switch operations          scale etc
```
