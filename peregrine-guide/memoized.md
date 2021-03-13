# Memoized

## Introduction

Memoized is a class which is used extensively in `peregrine-data` to handle results which can only be resolved asynchronously but which are static and so can be cached. It presents a simple async interface to the requestor while deduplicating pending requests and cacheing values.

Cacheing can either be "complete" or an LRU-based cache. THe main reason why memoized is used over a simpler cache is to reduce the number of repeated pending requests.

The class is polymorpih on `K`, they key and `V` the value.

## Creating a Memoized

There are two memoized constructors, `new` and `new_cached`. BOth take a _resolver_ callback. This is the method to, in each case, actually populate any missing data. The resolver takes two arguments, the _key_ (of type `K`) and a `MemoizedDataResult`. This callback should then set in motion the resolution process, ultimately calling the `resolve` method on the `MemoizedDataResult`. Keys can also b added with the `add` method.

## Implementation

Two classes from Commander are used in the implementation.

* `PromiseFuture` is a simple suture which can later be resolved with a value. These are returned to each caller.
* A `FusePromise` takes zero-or-more `PromiseFutures` and also has a method `fuse()` to set a result. From themoment `fuse()` is called all registered `PromiseFuture`s are satisfied with the value (which must be Clone). Any promises added later are also instantly satisfied with the same value.
