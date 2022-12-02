# StyleTree

## Overview

The StyleTree maintains the patterns and their associated styles in order that specific paths, ie individual leaf paths, can be looked up to see if there are any matches, and so any styles for that leaf. The only thing which makes that other than a non-trivial map lookup is the possibility of wildcards `*` (matching any one path jump and called an `AnyOne` in the code) and `**` (matching an arbitrary number of path jumps, called an `AnyMany` in the code).

This lookup could easily be a limiting performance issue in the browser (as it can be for CSS in web browsers) so the path specification and the algorithm chosen are a comporomise between performance and complexity. There are routes we could go down that could lead to expontential explosion in data size or compute time (as this is essentially an NFA->DFA problem), and yet without non-determinism the patterns which could be specified are probably not sufficiently powerful. So it is a bit of a tightrope.

Most of this potentially combinatorial explosion comes from AnyMany. In most cases, AnyMany would be used initially in a spec (eg `**/a/b/*/d`). To allow this, adds and lookups are actually performed reversed. For example, `a/b/c` is stored and looked up first through an arc `c` then `b` and finally `a`. The order makes no difference to the algorithm. However, it _does_ place an initial AnyMany as a suffix (ie `**/a/b/*/d` is stored as `d/*/b/a/**`). Such patterns can be treated as a prefix matching problem (rather than exact string matching as in the cases without a terminal `**`). So each node stores two staches of properties for lookups: exact and prefix stashes. Exact stashes are added to the output when a lookup string finishes at exaclty that node, prefix stashes are added even when "passing through".

For example, consider property `A` assigned with pattern `a/b/c/d` and property `Z` assigned with pattern `**/c/d`. In this case we will build a tree `root -> d -> c -> b -> a` and will have `A` stored in the exact stasth of the node `a`, but property `Z` assigned to the prefix stash of node `c`. When a lookup for `a/b/c/d` goes through the tree it will (correctly) pick up the property `Z` as it passes through node `c` and also property `A` from node `a`.

This leaves the implementation of `*` and non-terminal `**`. As the tree is cached, the number of lookups will probably closely follow the number of adds, so there's no real reason to bias our computation either during addition or lookup. If we put it at add we could do an NFA to DFA transformation which would increase memory and take CPU time but leave a tree with fast lookups (as at each node only one branch would need to be considered). If we put it in lookup, we would keep the tree as an NFA which will make adds fast and memory small, but mean lookups have to take multiple branches. The usual approach is the former wherepossible as lookups vastly outweigh adds. In our case that's not the context, so the latter approach is taken both for simplicity and to keep the footprint small.

Adds therefore are simple step-by-step adds to the tree. Lookups are recursive. When a node with an AnyOne is encountered, both the transition for the given letter *and* the AnyOne branch must be taken. For exmaple in the following, passing through branch at A with a path-part `a` would need both subtrees X and Y to be searched.

```
           O
          / \
         m   n
        /     \
       O       A
      ...     / \
             a   *
            /     \
           X       Y
```

Internal AnyMany nodes (ie ones which cannot be optimised away by the approach above) are even worse. Potentially any number of the remaining path components can be "eaten" by the AnyMany node (including none), so for each prefix of the remaining path components the tree must be searched with that prefix removed.

For example, if we have the following tree, and the pattern `m/a/b/c`, we need to search from node A with strings `a/b/c` and `b/c` and `c` to be sure to pick up `X` and `Y` (as well as `Z`).

```
           O
          / \
         m   **
        /     \
       Z       A
      ...     / \
             b   c
            /     \
           O       Y
           |
           c
           |
           X
```

This explosion could be catastrophic, expecially with multiple (non-prefix) inline AnyManys so they should be avoided or at least placed after an uncommon suffix (to mean lookup never reaches them)!

## Code

As in other parts of the code, the algorithm is divided into two parts by polymorphism to separate the core algorithm from the ephemera and to allow easy testing. The core algorithm sits in `pathtree.rs` and the wrapper in `styletree.rs` which also incldues a cache and performs inheritence of leaf properties from containers higher in the tree (a detail of how styles work in the genome browser).
