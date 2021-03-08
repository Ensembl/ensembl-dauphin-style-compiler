# Code Layout

This document is intended to help you navigate around the code.

## Backend Code

This module is written in python. It is the backend service. It transforms the data, `begs` files and other things the browser needs into payloads which are serialised in `cbor` for efficiency (which is very like JSON but binary and fast).

* `backend-server`

## Shared (command-line & browser)

Libraries (named `*-lib-*`) implement all the commands and have a cut-down version which goes into the style interpreter (browser) and a full version in the style compiler (command line) depending on acompile flag ("feature" in rust-speak). `dauphin-interp` is common to both the style compiler andthe interpreter.

To keep things modular, much of the core language ismoved out to the `lib-std` library, including things like assignment! You can't really get any working program without `std`, it's kind of non-optional.

`lib-peregrine` contains all the commands for actually communicating with the genome browser proper: to request data, decompress it, draw it, etc.

`lib-buildtime` contains things that don't end up in thefinal `begs` file because they're entirely executed on the compiler. These might be like macros in other languages. Things like looking things up in ini files, etc, go in here. This is a separate library so that you don't have to link it in the browser and so non of its dependencies are needed either.

* `dauphin-interp` -- common code for compiler and interpreter. 
* `daphin-lib-std` -- core language.
* `dauphin-lib-peregrine` -- genome-browser interface commands.
* `dauphin-lib-buildtime` -- commands which can only run at compile time.

## Style-compiler specific (command-line)

Most of the functionality of the style compiler sits in `dauphin-compile`.  The other crate, `dauphin` is just for top-level stuff like parsing arguments, reading and writing files, reporting things to the user, etc.

* `dauphin` -- command line
* `dauphin-compile` -- style compiler

## Browser specific (data retrieval)

Most of the browser crates are about data retrieval and processing, running the style interpreter, and so on.

`perergrine-dauphin` is the crate which embeds the style interpreter (aka dauphin) inside the genome browser (aka peregrine). It's a small crate which needs to be separate to avoid circular dependencies. It is simply responsible for taking a given `begs` program and running it.

`peregrine-dauphin-queue` is an even smaller crate which only exists for circular-dependency avoidance reasons. It contains the queues in and out of `peregrine-dauphin` for submission requests, data, and results.

`peregrine-core` contains all of the browser-side code concerned with retrieving data and `begs` programs, running them, building data structures, and so on. It's also responsible for requesting the right thing at the right time (given position and zoom level). The only things it doesn't really do relate to the browser itself (DOM, WebGL, etc).

`commander` is an async executor. Unlike javascript, where asyncs are built into the browser, in rust you get the language support but you have to poll the created asyncs to push them to success. While this normally would be irritating (though there are standard ways of doing this) this is actually lucky for us because we need an unusual async executor:

* we have two kinds of time "tick": clock time and browser frame. At any one time, one or more of these may be active and we want our callbacks to be enabled or disabled based on that (no constant polling when quiescent), and we also want to wait for these periods independently.
* we need to regularly drop back to the main browser loop: we can't handle busy-waiting just by spinning up a thread.
* we need priorities to batch up our requests. We have real-time threads but also threads which actually benefit from only running in idle time: the HTTP request threads. As we need to do our best to batch requests, having theissuing and responding be a low-priority task naturally does this for us.

Commander implements an executor meeting these specs. It's hairy but independent of any browser code.

* `commander`
* `peregrine-core`
* `peregrine-dauphin`
* `peregrine-dauhpin-queue`

## Browser specific (visual and web)

All drawing and browser interaction takes place in peregrine-web.

* `peregrine-web`

## Some data-structures

These are little crates which do some task that I couldn't find a generic library to do for me.

* `identitynumber`
* `keyed`
* `varea`

## Debugging and development

`blackbox` is only compiled in debug builds and creates a "phone-home" logging system for debugging.

`dauphin-egs-file-extension` is a vscode extension for pretty `egs` files.

`dauphin-interp-toy` is an interpreter independent of the web. This is useful for tinkering with the dauphin interpreter while keeping all the peregrine stuff out of the way.

`dauphin-test-harness` is used in unit tests for some common functions.

`peregrine-guide` is these files.

* `blackbox`
* `dauphin-egs-file-extension`
* `dauphin-interp-toy`
* `dauphin-test-harness`
* `peregrine-guide`

## Production Build Dependency Graph
Arrows point towards thing depended upon. Dependencies are also arranged "upwards" in the diagram. Dotted arrows `.....>` only apply when building style-compiler (ie for command line).

```
                                         v-----------\
        /-------------------> dauphin-interp<--------\\
       /     /---------------^   ^   ^     ^^--------\\\
      /     /       /-----------/   |        \        \\\
     /     /      /        dauphin-compile    \        \\\
    /    /      /          ..^.^  ^      ^.   |         \\\
   /   /        |        .:   :   |        :  |          \\\
  /   /         |       :   .     |        :  |           \\\
  |  |    dauphin-lib-std  :      | dauphin-lib-peregrine |||
  |  |         ^       ^ ^\:      |              ^ ^      |||
  |  |         |        \.: \     |              | |      |||
  |  |         |       : |  |     |              | |      |||
  |dauphin-lib-buildtime |  |     |              | |      |||
  |   ^          ^       |  |     |              | |      |||
  \   |   /------+-------+-/      |              | |      |||
   dauphin-------+-------+-------/               | |      |||
    ^    \-------+-------+-----------------------/ |      |||
BINARY           |       |                         |      |||
                 |       |                         |      |||
      /----------/       |                         |      |||
     //------------------/                         |      |||
    ///--------------------------------------------/      |||
   ///                                                    |||
  ///           identitynumber       blackbox             |||
 ///               ^                 ^   ^ ^              |||
///                |                 |   | |              |||
||| /-------->commander--------------/   | |              |||
||| |         ^    ^   ^                 | |              |||
||| |      .-'    /    |                 | |              |||
||| |     /      /     |                 | |              |||
||| |    /   ,--+-.    |                 | |              |||
||| |   |   /  /   v   |                 | |              |||
||| |   |  /   | peregrine-dauphin-queue-/ |              |||
||| |   | |    |  ^   ^                    |              |||
||| |   | |    \ /    |                    |              |||
||| |   | |     X     |        /-----------/              |||
||| |   | |    / \    |       /                           |||
||| |   | |    |  \peregrine-core------------> varea      |||
||| |   | |    |     ^  ^   \ \--------------> keyed      |||
||| |   | |    |     |  |    \                  ^         |||
||| |   | |    |     |  \     \-----------------+---------/||
||| |   | |    |     |   \----.                 |          ||
||| |   | |    |     |         \                |          ||
||| \---+-+-peregrine-dauphin---+---------------+----------/|
\\\-----+-+---/  / / ^          |               |           |
 \\-----+-+-----/ /  |          /               |           |
  \-----+-+------/   |         /                |           |
        \ \          | /------/                 |           |
         \peregrine-web-------------------------/           |
            ^          \------------------------------------/
          BINARY
```
