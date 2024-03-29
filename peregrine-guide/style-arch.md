# Design of the Image Style language

This document describes the design criteria for the style language. It is a little technically complex, but was written to justify the design chosen. This doucment can be skipped if that doesn't interest you at present. The actual compiler and its language are described in later documents.

## Introduction

The motivating factors in the design were:
* simplicity,
* speedy implementation,
* minimal effort, and 
* long-term stability and extensibility.

This document should be read in parallel with documents describing the language itself.

## Challenges

Developing a comprehensive, declarative, decade-stable data model for the style element is as much of a fools errand as developing one for the bioligy. CSS attempted a similar delcarative approach for a much simpler domain (and DSSSL and many others before it) and the result was a mess. This pushes us away from a declarative model towards an imperative one -- towards code.

Code has the flexibility to represent arbitrary transformations. But to implement an efficient byte-code interpreter or machine-level compiler is also a fools errand on a small project such as Ensembl. An intermediate approach is required: a language which allows rich composition of transformations and easy implementation of arbitrary extensions, but without the complexity of a full programming language.

## The byte-code problem and the dispatch trap

The poor performance of byte-code interpreters is often down to the slow speed of inter-instruction dispatch. That is, the buisness of an instruction can be disposed of efficiently, but the process of selecting the next instruction, various housekeeping concerns and so on, all taking place inbetween intstructions, dwarf the total time spent on the intstructions themselves. There are techniques to avoid this, but they are time-consuming, complex, and host-specific. We need our style language interpreter to be efficient because we are dealing with large data-volumes. We might typically be handling a dozen arrays of perhaps a few thousand elements. An ill-advised microsecond or two could sink our ship if we arenot careful, and we don't want to have to go to the effort of building one so watertight.

What can be done? The approach we take is to work on vectors as our fundamental unit of operation. Each instruction operates on the entire *vector* of starts, stops, lengths, colours, etc. In this way the task of an instruction is parallelizable and many fewer instructions are executed, leading to a much smaller inter-instruction dispatch trap. This is effectively the same approach taken by GPUs. Operations should fit within low-level CPU caches and theoretically would even be parallelizable.

## Branch-free conditionals

When working on whole arrays of starts, stops, colours, etc, there is always the case of certain subsets of the arrays needing to be transformed in one particular way. For example, we may have an array of gene colours which need transforming but only if they are positive strand, or of a particular biotype.

We attampt to preserve efficient array-based operations while allowing distinct handling of elements by accommodating *filter arrays*. These are boolean arrays which can be applied to transformation instructions to provide go/no-go to the instruction for each element. For example we can add some constant to some of the numbers in an array by supplying the array to be modified, the constant itself *and* an array of true/false members which give the elements to be affected and those to be passed over. These boolean filter arrays might be initially generated by, say, passing over the array and performing some test to each element. This allows branch-free conditional operations. It is essentially the same technique used in branch-free programming for security (but is not being used for security purposes here).

By supplying special syntax for the filter operations within the core language, we relieve each instruction of much of the burden of accepting filter parameters themselves.

## Loop-free execution

By removing both branches *and* loops we sacrifice much expressive power from the language. Initially this seems like it's a disadvantage, but the upside is that we gain a lot in terms of analysability in taking this route as we are no longer Turing Complete and so don't suffer from Halting Problem type issues. The execution time of the instructions can be easily analysed as the interpreter always proceeds in order from one instruction to the next. As long as each instruction always terminates, so will the program. If an instruction takes a certain time per array member, an estimate of the upper-bound of the size of the data will yield a good upper bound on the total execution time.

This analyseability is of more than theoretical interest. It allows us to do a number of practical things. 

* We can aggressively evaluate at compile time any instruction for which all its values known already. As a lot of style data will likely be pulled-in and modified at compile time, this saves a lot of work on the browser.

* Our support for composite structures (see later) makes analysis of dead-paths and register rewriting vital for efficient code execution.

* Register rewriting can be used to prevent unnecessary operations which various transformations would otherwise introduce.

* Analysis of execution times allows inter-instruction housekeeping (such as checking RTC for timeslice expiry) to be skipped for sequences of known-short instructions.

All of these analysis tasks are simple with loop-free, branch-free code, but almost impossible otherwise.

Given the design if, instead, we want to do anything resembling real computation, it should be done with custom opcodes not by attempting to shoehorn it into the core language.

## Structures, Unions, and Deep Vectors

Much biological data is one-dimensional but not all of it. We need to allow structures, unions, and arbitrarily deep vectors, and various nestings of these things. But how can this be done, while preserving the tight inner-loops of flat, one-dimensional arrays of simple types?

We support a syntax which appears to allow such arbitrary structures but which are ultimately represented by flat, one-dimensional arrays of simple types. The various structures that the language appears to support are naming conventions and transformations on array offsets and filters, hidden from the source author. For example, we may believe we have a gene object comprising a start and stop co-ordinate, and build such gene objects into an array. However, the reality is that we have two independent arrays of start co-ordinates and stop co-ordinates. It is this illusion which risks generating a large number of "dead" data paths in compilation, and which benefits from the analysis made possible by loop-free execution (described above).

## Compile-time data blending

For good reasons, many people will not wish to engage with the style language. It will be seen as obscure and cumbersome. To this end we are rescued again by the extra analysability at compile time. Certain constants, colours, fonts, labels, etc, we might anticipate being regularly changed. We can allow such constants to be pulled from, for example, ini files, or REST sevices at compile time. Once these things are changed, a simple recompile will alter the code in a way just as efficient as if the source had been modified by hand.

# Summary of Image Style Code

By using a branch-free, loop-free language we retain much of the *image style* expressibility which we actually need in terms of composition, transformation, and rich data-strcutures, without falling into the big, time-consuming, complex traps of implementing a programming language which attmpts to do so efficiently for large data volumes. The massively increased analysability of the resulting, very restricted code actually allows us to support features in the source to "fake" much of the missing branching and deep structures for the end programmer, without sacrificing speed.

In terms of long-term replacability (one of our design goals), either the input or the output of the style language compilation for a track (or both) could easily be captured and replaced either by a new processor in some other language (either more powerful or less) or captured as data for static delivery.

## Data-producer workflow

Most people will probably deal with style language as black-box gobbledigook, and this is fine.

* A data producer would typically bundle their data along with some style-code which they pull from a common collection without delving into the code at all. Most people will be developing tracks very much like all other existing tracks, so this is fine. The codebases just need to be clearly labelled "gene track*, *variant track*, *GC wiggle*, *read stack* etc, and a very few such source files should cover the vast majority of our data.

* Occasionally, a track will be developed with custom colours, sizes, fonts, and so on. In this case an existing source is still pulled as a "black box" from a common repository, but an ini file may be customised.

* Sometimes genuinely new data will come along. In this case the data-producer (or a member of the webteam on their behalf) develops a new track by writing custom *image style* code. In most other browsers such a change would require model changes and new code. With the changes here only requiring *image style* changes, the track will be backwards- and forwards-compatible with the genome browser and the style archived along with the data and available for other tracks.

* In very rare cases some new drawing primitive will come along (eg translucent rotating cube or something!) In this case, the style and data will be forward-compatible but not backward. The new data will display on sites supporting the new primitive along with all old data but the new data cannot be retrospectively displayed on old sites.
