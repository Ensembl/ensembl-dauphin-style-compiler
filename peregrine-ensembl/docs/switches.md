# Switches

Dots here mean path components in the actual api. ie `x.y.z -> ["x","y","z"]` because the former looks less ugly in a document.

## Specified by chrome

In the following `GENE-TRACK` is one of `gene-other-rev`, `gene-pc-rev`, `gene-other-fwd`, `gene-pc-fwd`, `focus`. `TRACK` means one of `GENE-TRACK` or `gc`, `variant`, `contig`. These will be replaced with proper ids in time.

* `track.TRACK` (bool) -- track on/off
* `track.TRACK.name` (bool) -- track label
* `track.GENE-TRACK.label` (bool) -- trackgene label
* `track.GENE-TRACK.transcript-label` (bool) -- track show transcript label
* `track.GENE-TRACK.several` (bool) -- track show 1 vs 5
* `track.focus.item.gene` (string: GENE-ID) -- focus object is a gene; it is this one
* `track.focus.enabled-transcripts` ([string]: [TRNS-ID]) -- show these transcripts

## Consistency

You probably always want these hardwired on. They're just there to avoid having to hack the code mainly. Unless indicated otherwise, should be set to `true`.

* `ruler` (bool) -- show ruler
* `track` (bool) -- show tracks
* `settings` (bool) -- use settings
* `track.focus.item` (bool) -- the focus track has an item in it

## Specified by config from within genome browser at certain scales

(No need to worry about these re integration)

* `scale.shimmer` (bool) -- very zoomed out contigs, need special treatment
* `scale.no-labels` (bool) -- render as transcripts but don't show labels due to scale
* `scale.no-letters` (bool) -- render as base-pair boxes but too zoomed out to draw letters

## Deprecated

* `focus.gene.GENE-ID` (bool) -- older, deprecated form of `track.focus.item.gene`.
