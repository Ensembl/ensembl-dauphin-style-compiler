# Notes from building the Regulation track

## Drawing shapes in three separate lanes
The purpose of this step was to discover how to draw shapes inside of different "leaves", and position the leaves relative to each other in a way that they would stack one under the other resembling three distinct lanes along which we would draw different regulatory features.

The hardest part about this exercise was the undocumented nature of the style language used inside of the eard files. As we can see in `containerstyle.rs` Rust file, there are four types of "allotment" — `stack`, `overlay`, `bumper`, and `wall`. From these names alone, I expect `bumper` to cause one shape to bump an overlapping shape such that it goes under the first one; `overlay` to let shapes intersect with one another; and `stack` to always put one shape under the other. I do not know whether this interpretation is correct. I also do not know what `wall` does; or what is the point of the `bump-height: none` rule.

The first attempt (unsuccessful): create three different "leaves" inside of the `regulation/main/main/lanes` leaf, and draw shapes directly inside:

```
style!("""
    tracks/track/regulation/main/main/lanes/ {
        type: stack;
        priority: 1;
    }

    tracks/track/regulation/main/main/lanes/first/ {
        min-height: 10;
        priority: 0;
    }

    tracks/track/regulation/main/main/lanes/second/ {
        min-height: 10;
        priority: 1;
    }

    tracks/track/regulation/main/main/lanes/third/ {
        min-height: 10;
        priority: 2;
    }
""");

let top_leaf = leaf("tracks/track/regulation/main/main/lanes/first");
let middle_leaf = leaf("tracks/track/regulation/main/main/lanes/second");
let bottom_leaf = leaf("tracks/track/regulation/main/main/lanes/third");
```

Surely that would be similar to how one would do this in CSS? But this did not work; shapes kept jumping around when drawn at overlapping coordinates, instead of getting drawn one below the other:

https://github.com/Ensembl/ensembl-dauphin-style-compiler/assets/6834224/7d4ebd52-66ae-41af-8b34-c20f9b46e93c

What solved this was nesting another leaf inside of `/regulation/main/main/lanes/first` etc.:

```
let top_leaf = leaf("tracks/track/regulation/main/main/lanes/first/content");
let middle_leaf = leaf("tracks/track/regulation/main/main/lanes/second/content");
let bottom_leaf = leaf("tracks/track/regulation/main/main/lanes/third/content");
```

So that the final leaf structure became like so:

![image](https://github.com/Ensembl/ensembl-dauphin-style-compiler/assets/6834224/eb90edda-0949-4c75-9ac4-a20f66b1f8cf)


## Running interpreter for the regulation program in the console
A program can be run in the terminal using the peregrine-eard interpreter (needs compiling the rust first). In order to run a program, the interpreter needs to be provided with:

- The name of the program to run.
- A mock data file with the data that the program requires to run. At a minimum, such data file needs to have a `__request` field containing bp range at which the program executes, and a `__settings` field. See `backend-server/egs-data/test/ruler.json` file for a minimal example, or `backend-server/egs-data/test/gene.json` for a much more involved one.
- Path to the compiled eard

Example:

```bash
# starting from ensembl-dauphin-style-compiler directory
../peregrine-eard/interp-cli/target/release/eard-interp-cli -p ensembl-webteam/core:regulation:1 -r ./backend-server/egs-data/test/regulation.json ./backend-server/egs-data/begs/render16.eardo
```
