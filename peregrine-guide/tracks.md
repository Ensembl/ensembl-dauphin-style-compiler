# Tracks

When we casually talk about a track we mean a number of different things. In this code specific terms are used to try to distinguish them.

* An **allotment** is an area of screen alloted to displaying some data. Allotments come in different **allotment schemes***. The most familiar such scheme is the horizontal stripes of space which are often called tracks. Other schemes include things like overlays, etc.

* A **track** is a source of data (in the form of raw data plus style language). A track can (and typically will) draw to multiple allotments. For example, the gene trackmight draw to five: the focus allotment and the two pc and non-pc gene tracks on both strands. The style language is responsible for allocating track data to different allotments as it sees fit.

* A **switchable** is something which can be switched on and off. If at least one switchable is non-off, a *track* is loaded and run to distribute its data to various allotments.

## Overview

```
UI-interactor                    Displayed region
    |                                |
    v                                v
Switchable1 --> +--------+ ----> Allotment1
Switchable2 --> |        | ----> Allotment2
Switchable3 --> | Track  |       ...
Switchable4 --> |        |
...             +--------+
                     ^
                     |
                   data
```

for example:

```
UI-interactor                    Displayed region
    |                                |
    v                                v
Cog switch1 --> +--------+ ----> Focus
Cog switch2 --> | Gene   | ----> protein-coding forward
Panel icon1 --> | Track  | ----> other forward
Panel icon2 --> |        |       ...
...             +--------+
                     ^
                     |
                   data
```


## Track Authorities

Track authorities are backend services which answer questions about allotments, tracks, and switchables. As with a stick authority, the track authority includes a *bootstrap* program. The bootstrap program is run when the authority is first encountered and allows the authority to add its data to various internal services. Track authorities can also assign priorities to allotments.

Track authorities can be specified at boot time or by stick authorities.

## Switchables

Switchables are string keyed and have an arbitrary json-like value. The value `null` is distinguished among such values to indicate the associated track need not be run, and its data excluded.

Switchables are defined by track authorities in their bootstrap program and at the point of definition are associated with a channel and track name.

Whenever a switchable is non-null, the track is run. The contents of the switchable (and all other switchables associated with the track) are available for decisions as to rendering.

## Tracks

A track is essentailly a style program and a data source. It is retrieved and enabled by a relevant switchable. When adding data, the track program requests allotments from an allotment scheme. The schemes available are fixed for a given version of the genome-browser. The scheme string is freeform within each scheme to allow complex specification of the exact allotment requested.

## Allotments

Allotments are bundled in schemes. The most useful is the "band" scheme, representing what are traditionally seen as visual "tracks". When an allotment is requested by a track, the **name** of the allotment is specified which allows the priority scheme specified in the bootstrap to be applied.

## Cog Set

A cog-set completes the circle from allotments back to switchables. A user believes they are "configuring the strack" (without having to grasp the potential subtleties). A cog set is merely a specified sub-namespace of switchables partitioned by allotment name.

## Allotment Priorities

**TODO**

## Multiple Transcripts

**TODO**
