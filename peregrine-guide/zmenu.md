# ZMenus

## Data Model

ZMenus are added one-by-one by dauphin. If their boxes overlap then they compose into a single Visual ZMenu, however all the generating code treats them as separate entities.

A single VIsual ZMenu:
```
+--------------
| First ZMenu ......
| Still first ZMenu
|
| Second ZMenu......
| Still second ZMenu
| ...
```

A `ZMenuFeature` is structured a little like an old-school image gallery, being a sequence of items which are arranged horizontally, a bit like inline-blocks, with optional "breaks" maybe forcing a new line from time-to-time, with other newlines added as needed by wrapping. Each of those items is called a `ZMenuBlock` and the separator a `LineBreak`. The enum `ZMenuBlock`/`LineBreak` is named `ZMenuSequence` and a list of those make a `ZMenu`.

Possible visual layout of a zmenu:
```
+-----------------------------------------+
| ZMenuBlock1   ZMenuBlock2   ZMenuBlock3 |
| ZMenuBlock4   ZMenuBlock5   ZMenuBlock6 |
+-----------------------------------------+
```
The seven corresponding ZMenuSequences:
```
ZMenuBlock1 ZMenuBlock2 ZMenuBlock3 LineBreak ZMenuBlock4 ZMenuBlock5 ZMenuBlock6 
```

A `ZMenuBlock` should "run" like a sequence of text and not be offset with internal structure. However it has structure to the extent that items can be styled inline as italic, links, etc. It's therefore composed of a sequence of `ZMenuItems` which include text
(literal or placeholder) and markup flags. Each piece of text is a `ZMenuText` which is either literally text or a placeholder for data.

Because, rightly no one really cares about this internal structure a template mini-language constructs these from a simple string. Items are enclosed in `[]` and line-breaks indicated with a `/`. On and off markup strings are in `<>`...`</>` (like XML) and template strings are in `{}`. Any can be backslash-escaped. Templates are the only way to create these things. The above internal structure exists to simplify operations on ZMenus.

(`ZMenuToken` is a lex token used temporarily internally during parsing and can be ignored).

## Expanded Implementation

Each type of Zmenu potentially expands to a very large volume of data which, as with the WebGL etc, we should avoid expanding from its compact, generated form if possible. To this end, when rendered `ZMenus` are converted to `ZMenuGenerator`s. These allow the creation of proxy elements (`ZMenuProxy`) for the first, second, third, ZMenu etc of the given template and model. At any time, the `value()` method of `ZMenuProxy` can be called to retrieve the calue for that proxy in O(1) time.

The generated ZMenu data-structure shadows the `ZMenu` structure above but has no template members, all strings being resolved.

This code is within `zmenufixed.rs`.
