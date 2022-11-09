# Track Payload

This document will only make sense if you have read `tracks-background.md` at least in overview.

There is a payload with a certain schema, which travels from the backend to the frontend, and which is responsible for binding style scripts to the switch tree. In practical terms this is track management; it configures the genome browser to retrieve certain data and put it on the screen when the UI turns on certain settings.

The genome-browser backend would be expected to generate these payloads, and to use the Track API (somehow) as its data-source to do so.

The payload contains the following (explained later):
1. a program name;
2. trigger switches;
3. settings bound to switches;
4. fixed settings;
5. chromosone tags;
6. scale start, end, and step.

Every response from the genome browser can contain zero or more track payloads. Whenever they are present they must be acted upon by the genome browser. However you are almost exclusively likely to see them:

1. on boot, for basic configuration and for common tracks;
2. on expansion.

Expansion is a lazy-loading process which is designed to deliver payloads from on-demand sources (such as a track API), and is described in more detail later.

## TrackPayload Contents

### Program name

The program name is the name of a script to run. It comprises two strings and an integer (group,name,version). But any difference in any one of the three is considered a completely different program name. The decomposition only exists to make managing program names easier.

The "script to run" (program name) will correspond to something like "gene", "variants", "GC, and so on and drawn from a relatively small set. It will probably mainly depend on the way data is displayed but maybe also data source (if very different between different instances).

### Trigger switches

Trigger switches are paths in the switch tree which, when true, should cause the scrip to run. This basically means "how is the track turned on". For example it could be that a trigger is `["tracks","regulation"]`. If the UI sets that switch to true, then this track is activated. Note that there can be multiple triggers. In this case the program is only run once. THe values of these are basically determined by how the UI wants to turn on a track.

### Settings bound to switches

Sometimes a script will need some settings. These settings are varied and might include:
1. the id for a data-source;
2. the id of a box in which to draw a shape;
3. various switches altering visual appearance.

In some of these cases (for example, togglable labels), we want this to be determined by a UI setting. So the setting is bound to a switch mapping the setting (a string) to the switch (a path). For exmaple the `show-labels` setting for some track might be bound to `["tracks","regulation","labels"]`. The available settings are determined by the script used and the switches they map to depend on the UI team's preference.

### Fixed settings

Sometimes a setting just needs a fixed value for a given track. For example, many of the IDs used to retrieve data will be fixed in this way. For example, two gene tracks, might use the same program name and run the same script, but have different endpoints for their data, maybe different settings (eg colour).

### Chromosome tags

Some tracks are only available on some chromosomes. For example, only some species have variation data. However, it would be unfeasable to name all the chromosomes and tracks, creating a horrible O(n^2) problem. Instead, chromosomes (sticks) can be assigned "tags". Whenever a chromosome is used for the first time there is a request to retrieve its "tags" (along with its size and other necessary information). A tag might be "#has-variation" or "#grch37" or anything! It probably makes sense before there are too many to manage this namespace formally.

Each track has a tag predicate comprising tag-names and/or stick names, arranged into a predicate with and, or, and not. For example `#grch37 & !#mito`. This is just a string as far as we're concerned in this payload.

### Scale start, end, and step

Tracks only make sense at certain scales. A scale is approximately 2^n base-pairs per screen (the exact position of the changeover depends on a number of external factors). So scale "20" means "around 1,000,000 bp on the screen", whereas scale "10" means around 1,000, and scale "5" means only around 32 bp on the screen. start and end are inclusive.

Step is a complex parameter which probably shouldn't be in the payload but set by the program (but this isn't possible right now). If the step is `1` then at each scale (corresponding to a factor of two) then the program is rerun. However, for efficiency, the script can only be rerun every step scales. If a script has enough, precise-enough data for all scales this saves a lot of computation. The value of step should be on the advice of the script writer.

The overall shape of the tracks payload on the wire can be considered to be something like:

```
{
    "program_name": ("ensembl/webteam","gene",1),
    "triggers": [
        ["tracks","pc-gene-fwd"],
        ["tracks","pc-gene-rev"]
    ],
    "switches": {
        "labels": ["tracks","pc-gene-fwd","labels"],
        "track-name": ["tracks","pc-gene-fwd","track-name"],
    },
    "settings": {
        "data-source-id": "ensembl-gene-standard",
        "target-box": "gene"
    }
    "tags": "!#has-no-genes",
    "scale": [10,15,2]
}
```

The payload is actually an object with fields to set and is emitted compressed, so the above JSON doesn't occur anywhere, but it gives a clear idea of the underlying schema.

The values in the above example would come from the following sources:

* `program_name`, `triggers`, `switches`, `settings`, `tags`, `scale`: these correspond to fields in the track payload object with corresponding setters (with very similar names) and don't really "exist".
* `["tracks","pc-gene-fwd"]`, `["tracks","pc-gene-rev"]`, `["tracks","pc-gene-fwd","labels"]`, `["tracks","pc-gene-fwd","track-name"]`: these are switches and would be determined with the UI team to correspond to their management of the switch tree.
* `labels`, `track-name`, `data-source-id`, `target-box`: these are created by the script writer and they will tell you what can go in there and what it does.
* `!#has-no-genes`: `!` means not: `#has-no-genes` would be drawn from some managed set by people uploading chromosomes/species to flag various properties of the species/chromosome which affect data availabilty.
* `[10,15,2]`: on advice from the script author, the scales to show the program at.

Note that this script is run when *either* pc-gene-fwd or pc-gene-rev is on.

## Challenges integrating with the track API

* scale -- how to store the fact that there may be multiple independent payloads for a given "track" at different scales

* composite tracks -- how to represent composite tracks which may have multiple independent payloads for each part, *or* may have a single payload with lots of settings, *or* some combination of the two, depending on the best way to write the program.

* decomposite tracks -- some "tracks" actually create multiple visually-independent tracks with multiple triggers and different settings for each, which we might expect to have multiple track api entries. (eg the gene scripts create four).

composite and decomposite tracks are hidden from the UI by the switches mechanism, but the representation in the track API might be a challenge.

Note that what we would need is just some algorithm implemented in the genome-browser backend which generates these payloads based on information from the track api. There doesn't need to be a one-to-one mapping.

See `boot-tracks.toml` for the current, temporary way of creating these payloads. Though using toml is a temporary solution, the data used is real.

* a challenge for the UI teamp: how to manage the switch tree -- up to the UI team

## Expansions

In effect, track payloads attach as limpets to parts of the switch tree, causing things to happen when a switch is set. But what about when the number of tracks is too large to deliver all the payloads at boot time? This is what expansions are for. Expansions are another schema-based payload which comes from the backend.

Expansions also attach to parts of the switch tree. Whenever the UI first ventures into part of a tree to set a setting, before that can happen, any attached expansions are run. When an expansion is run it contacts the backend and the backend can provide any new track-payloads to insert into the tree. These track payloads are added _before_ the setting is applied and so may well be instantly triggered!

An expansion has a name (a string used as an ID) to provide when "phoning home" to give the backend some context as to the expansion triggered. It also takes a backend-namespace (a string-pair identifying the backend to use when responding, used throughout the genome browser) and the places to attach the expansion.

The idea is that though it may not be possible at boot time to attach all tracks, the expansions can be attached. Then any remaining tracks can come along, in dribs-and-drabs, lazily loaded just before a switch is set. In theory an expansion can even add further expansions further down the tree.
