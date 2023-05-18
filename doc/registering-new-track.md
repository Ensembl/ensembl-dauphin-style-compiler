# Registering a new track
Below are requirements to register a new track through the traditional Python channel.

## Add the track to the track manifest
Genome browser needs to be informed that a track exists. Currently, this is done statically, by adding track data to a toml file. For version 16, this file is `backend-server/config/boot-tracks/boot-tracks-16.toml`.

### Minimal required track metadata
Track declaration must include:
- Track identifier (name of the eard program that draws the track)
- The scales at which the track is displayed; formatted as `[start_scale, end_scale, step_for_redraw]`
- Triggers to show/hide the track

Example: for a track that is drawn by program `new-track`, which we want to draw at all scales, and want to instruct the genome browser to recalculate it at every scale, the metadata block will be:

```
[track.new-track] # set up an internal tree node, in this case for triggering an eard program "new-track"
include = ["general"] # set program name prefix/suffix (imported from another toml at line 2)
scales = [1,100,1] # rerun the program at every zoom level between 1-100
triggers = [["track","new-track"]] # set a trigger for UI to run the program
```

CAUTION: the smaller the third parameter in the `scales` array, the more often genome browser will redraw the track, the more compute-intensive the track becomes. The larger the step, the better for performance.

> DAN'S WISDOM: Tracks only make sense at certain scales. A scale is approximately 2^n base-pairs per screen (the exact position of the changeover depends on a number of external factors). So scale "20" means "around 1,000,000 bp on the screen", whereas scale "10" means around 1,000, and scale "5" means only around 32 bp on the screen. start and end are inclusive.
> Step is a complex parameter which probably shouldn't be in the payload but set by the program (but this isn't possible right now). If the step is `1` then at each scale (corresponding to a factor of two) then the program is rerun. However, for efficiency, the script can only be rerun every step scales. If a script has enough, precise-enough data for all scales this saves a lot of computation. The value of step should be on the advice of the script writer.

### Track settings metadata
A track will likely have settings configurable at runtime through commands sent to the genome browser. These should be registered in the boot manifest as well. For example, a declaration that the track with an id of `new-track` responds to the `name` setting, will look like this:

```
[track.new-track.settings]
name = ["track","new-track","name"]
```

The above attaches a setting called `name` to a subnode of `new-track` and wires it to a switch `["track","new-track","name"]`.
With that, the web client can assign a value to the switch (e.g. set it to `true`, see below), the eard program `new-track` can then read this value from setting `name` and react to it (e.g. draw a label in the track).

### Problems with the approach
- The track manifest is static. Currently, it lists information about all available tracks. This will not be sustainable going forward. Expect the instructions above to change.


## Add a program for drawing the track (or use an existing one)
Track drawing programs are written in a special domain-specific language and saved as `.eard` files. A drawing program can be split into multiple files; but the entry point into a program will be the file with a program declaration in the top.

Example:
```
program "ensembl-webteam/core" "test-drawing-program" 1;
```

> DAN'S WISDOM: Every eard script begins with a program line. This program line names a program. The name is used by every other part of the genome browser code (for example the track payloads) to refer to the script (in order to find it and to run it at the appropriate time). The name has three parts. The first part is a string which describes a set of people or a project which should have their act together to name things consistently. The second part is the name itself. The third is a version. The parts of the name are always processed together and the code doesn't break into the three parts: it always treats them as a unit. They're there to help you be consistent. There's no need to increment a version with every fix or change: it is there for if you want two versions to coexist in the system concurrently.
> There are two ways of including another file in eard which differ only in how they go about finding the file. `refer` is for internal stuff -- library headers, etc -- and are conventional names for things buried deep inside the compiler. The docs for a function in the standard library it will tell you what you need to add to the top of your file. On the other hand, `include` is for files you write yourself, relative to the current file. This is useful for writing helper functions/procedures for tracks which share some visual element, or for data endpoints which have a lot in common.

### Example of a minimal viable program
Suppose a new program is created in `backend-server/egs-data/egs/v16/new-track/new-track.eard`. It will include its id in the track summary report; will be able to display its name; will put a horizontal line and a rectangle on the screen, and will print a message to web browser console.

```
program "ensembl-webteam/core" "new-track" 1;
refer "libperegrine";
refer "libeoe";
include "../common/track-common.eard";
include "../common/track-style.eard";

/* 
 * Setup styles.
 * First, we are enabling common track styles, and then adding some custom styles for new track.
 * Note that the instruction to report track id in the track summary is part of the styles.
 */
/*  */
track_styles();

style!("""
    tracks/track/new-track/ {
        min-height: 100;
        priority: 990;
        report: "track;switch-id=new-track";
    }
""");

/* Make sure the name of the track can be printed */
draw_track_name("NEW TRACK","name",leaf("tracks/track/new-track/title/content"));

/* 
 * Even if you messed up the drawing code and can't see anything on the screen,
 * you can at least print stuff to the console to make sure your program is running
 */
print("hello world!");


/* 
 * Draw a 1-pixel-high rectangle across the whole viewport
 */
let test_leaf = leaf("tracks/track/new-track/main/background/content");
let paint = paint_solid(colour!("#d0d0d0"));

rectangle(
  coord([0],[0],[0]),
  coord([1],[1],[1]),
  paint,
  [test_leaf,...]
);

/* 
 * Draw a 5-pixel-high rectangle between 32357710 and 32357730 bp
 */
let test_leaf = leaf("tracks/track/new-track/main/main/content");
rectangle(
  coord([32357710],[5],[5]),
  coord([32357730],[10],[10]),
  paint,
  [test_leaf,...]
);

```

## Optional: Provide metadata about your program
_NOTE: this step seems to be optional; we have tried skipping it, and nothing bad happened. However, it gives you a chance to describe your program, if for nothing else then at least for documentation purposes._

Look around the `backend-server/egs-data/begs/specs16` directory. Notice that there are a lot of toml files there, each named as corresponding top-level program. If you inspect any of these toml files, you will find that they describe the settings of the corresponding program, with explicit default values. You can do the same for your new program by creating and filling in a `new-track.toml` file in that directory. When you are done, include reference to this file in `programs.toml` in the same directory.

## Include new track drawing program in the compilation script
The source code of track drawing programs needs to be compiled, which is currently done by the `backend-server/build-begs.sh` script. Inspect it to see how other programs are passed to the eard-compiler, and add yours.

NOTE: The compiler for track drawing programs resides in the `peregrine-eard` repository. You will need to have `peregrine-eard` and `peregrine-eachorevery` repositories available locally (both require Rust, and their source needs to be compiled; see the main `readme.md` file at the top level of this repo for instructions), or use a docker image with `peregrine-eard` already compiled. See how the path to `peregrine-eard` is referenced in the `build-begs.sh` script.

## Use the track
- Compile track drawing programs by running the `backend-server/build-begs.sh` script.
- Have the genome browser server running (see the main `readme.md` file at the top level of this repo)
- Initialise the genome browser on your web page, and pass it a command to show the track. If we take the `peregrine-generic/index.html` as an example, the command will be `genome_browser.switch(["track","new-track"], true)` to show the track; and `genome_browser.switch(["track","new-track", "name"], true)` to toggle the `name` setting of that track to `true`.
