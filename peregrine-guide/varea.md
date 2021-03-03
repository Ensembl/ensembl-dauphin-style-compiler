# Varea

## Introduction

Varea is a data-structure for storing data indexed by a region of n-dimentional space, including discrete values and ranges. It can be seen as a little database lookup library.

## Main API

THe main storage object is the `VareaStore`. This stores items of polymorphic type `T`, indexed by `VareaItem`. There are no constraints on the type of `T`.  A search can be created for later sexecution by `search_tiem()`. As with addition, the query is passed to this method as a `VareaItem`.  `search_item()` creates a `VareaSearch` object which can be executed, when required by the `lookup()` method on `VareaStore`. This returns a `VareaSearchMatches`, which is an iterator on `T`. It also includes a `get_id()` method which returns the `VareaId` of the most recent match. 

Before looking up, `VareaSearch`es can also be comdined with and, or, all, and and-not compound queries, `AndVareaSearch`, `OrVareaSearch`, `AllVareaSearch`, and `AndNotVareaSearch`.

When an item is added, its `VareaId` is returned along with a `VareaItemRemover`. The sole mthod on the latter, `remove()`, removes the object.

`VareaItems` are created with no arguments and are subsequently constrained by specifying values for axes via its `add()` method. Axes are string-keyed and accept an implementation of the `VareaIndexItem` trait as a value.

The library provides two implementations of `VareaIndexItem`: `Discrete` and `RTreeRange`.

`Discrete` takes a slice of values which is of some key-like polymorphic type (clonable, hashable, eq and static). It is for axes when values may only come from some discrete set (eg enum values). Any overlap between the items in an add and lookup will cause a match.

An `RTreeRange` represents a continuous range interval of values. Any overlap between submitted and retrieved ranges will cause a match.

## Internal Implementation

Because of the genericness of the service, the internal API is a bit of a maze. Internally, all items get given a `VareaId`, which is a simple `usize` and is allocated by `VareaStore` on addition. This is used to separate off the polymorphic payload into a B-Tree in `VareaStore` itself, and those payloads play no further role in the implementation.

Searching is implemented by implementors of the `VareaWalker` trait. It has a single method, `next_from()`, which returns the next matching item, if any, starting at the `VareaId` given in the arguments. Boolean conjunctive operations are performed by iterating through sub-walkers and calling the `next_from()` method appropriately.

`VareaAxis` is a concrete type for which there is one instance per axis string in `VareaStore`. On addition, for every axis added to the `VareaItem`: the `VareaAxis` `add()` method for that key is called with the newly-assinged `VareaId`. On building a search, the `lookup()` method on all `VareaAxis` values supplied are combined with an and conjunction to build the search.

`VareaAxis` itself is a trivial wrapper over the axis-type polymorphism. Every axis type (currently just `Discrete` and `RTreeRange`) are required to return a unique, non-changing string for that type called the `factory_id()`. `VareaAxis` contains a string-keyed has with the type `VareaIndex` as its value. *Note:* a `VareaIndex` represents an entire axis of this type and is distinct from `VareaIndexItem` in the public API which represents a *single entry* in such an index. If an index doesn't exist the `make_index()` method of the passed `VareaIndexItem` is called and expects a `VareaIndex` in return; that is an item is expected to have a means of generating an empty container of the type in which it will fit.

The `VareaIndexItem` trait is relatively trivial. As well as the methods mentioned above it is only expected to implement `into_any()` and `as_any()` to allow the polymorphism in `VareaAxis`. All the heavy-lifting is done by the `VareaIndex` trait and its `add()` and `lookup()` methods which are called with the items concerned.

### Dicrete Walker

The discrete walker covers discrete values. THe index is simply a HashMap of Sets of VareaIds in `DiscreteIndex` which implements `VareaIndex`. Adding adds the `VareaId` to one or more such sets, and lookup invovles taking a reference to one. Should multiple such values be specified, the or conjunctive walker is used to provide a single, combined walker.

### RTreeWalker

The RTree walker implements ranges. `VareaIndex` is implemented by `RTree`. Each `RTree` stores data in a number of `RTreeLevel`s. Each level handles a different range of object sizes. An object lives in exactly one level, so the add method simply chooses the correct level and then delegates to that level's `add()` method. Lookup iterates through all levels, calling their `lookup()` method, the combined methods being or-ed to find matches on all levels.

Intervals are semi-open.

Each `RTreeLevel` is divided into a sequence of `RTreeNode`s. Each `RTreeNode` represents a bucket of some fixed size appropriate to the level. Again, `add()` delegates to the appropriate node, whereas `lookup()` or-s all matches. This lookup method is rather inefficient as it prior-constructs big data structures in memory if there are many entries in a range: a way could probably be devised to do this on the or-ing stack without prior construction. Each `RTreeNode` is a map from `VareaId` to an `RTreeRange`.

Levels are chosen with a given "scale" which represents the binary-log of a size. For example, level scale 2 is 4 units long, level scale 3 is 8 units long, level scale 4 is 16 units long. Each `RTreeNode` in a level of a given scale is the size specified by the scale and are all appended. An item can only be added to a level where it fits entirely within a single node. 

An `RTree` contains only certain scales, specified by its merge argument. For example merge 4 only has scales 0, 4, 8, ..., and therefore node sizes 1, 16, 256, .... Currently, this value is hardwired to 4 in the constructor.

To determine the correct level to add an item to, `correct_level()` is called. First the scale of the line, were it 0-aligned is determined. Then that scale is rounded *down* to the appropriate merged scale. For example, a range of length 20 is determeined to be scale 5 (less than 32) and then merged to 4. Next the size of items in the chosen scale are calculated (in our example, 16). Finally, unless both ends fit into the same bucket at this scale, the next scale up is chosen (effectively turning the round-down into a round up).