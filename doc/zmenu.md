# ZMenu Payload Top-Halfs

## Scope

This document covers the decision to have the content of the top half of the zmenus sent by the genome browser app rather than it sending identifiers and have the rest of the application derive the contents via thoas.

For example, when a used clicks on a gene, a payload includes a string containing the ensembl-id, a string containing the biotype, a string containing the strand, and an arrangement of these strings for the application to render, and so on, rather than merely sending the identifier.

## Importance

Superficially this choice seems strange and anomolous, but it's a very important part of the overall design of the genome browser app, and it is an important to implement is as a part of that design being realised.

If it becomes impractical to do this from the application end, then the genome browser cannot effectively implement a lot of the motivating examples and use-cases, and a lot of its implementation just becomes irritating complexity.

So this document tries to comprehensively describe the use-cases to show that this is motivated.

## External Context

The architectural decisions are based on past experience of where complexity and work arises in the project and where mess tends to accumulate.

The relevant external context includes:

* The "correct" biology, and what we want to display changes incrementally over time. For example biotypes are added, merged and renamed, new fields such as IDs, and links between entities are added, and so on.

* Different species and kingdoms of life have different resources available, and even different relevant biology (prokaryotes have no exons, for example), some xrefs are not available for certain biotypes etc.

* We often have "wonky" versions of our main track types: LRGs, RNASeq genes, and soon are gene tracks "but different": different colours, different ZMenu contexts, and so on.

* Sometimes we have to keep old data formats available longer than we would otherwise like (in terms of code cleanliness).

* Any external desire to implement new track-types is tempered by the difficulty in implementing them: firstly, at all, and secondly, efficiently. The visual display is slow and irritating to use and slow.

## Implications

### Organisation around Components, Not Data

In the current codebase, code is arranged around the relevant component, for example the zmenu or each of the various kinds of wisual track. Within that code, there are lots of conditionals concerning the particular data in question, its presence or absense, flags for particular sub-varieties,. special cases, and so on. For larger changes there's an only partially-effective plugin system where code can be replaced in bulk, usually by a clone-and-hack type change, which tends to drift without considerable work. 

Effectively much of this code is the "business-logic" for ensembl. Effectively we have different "businesses" using our application (tracks, species, etc), each with their own logic. But rather than collecting each business's decisions together as the primary way of organising the code, we arrange by compoenent and a large number of flags and conditions for each case in the relevant component.

### Spread of Changes around Large Codebase

When we need to add support for a new kind of data track or a new variety of an existing track (historically such as LRGs or transcript "qualities", for example) we have to make changes throughout the code base which requires familiarity with it, and effectively restricts the change to just a few developers within the webteam: the code isn't dabbler-friendly. In many other applications that may not be important or even a benefit, but for science in general and ensembl in particular, scientists dabbling in development are important innovators. When control of the "black box" has been handed over (for exmaple, with innovations like track-hubs) the result has been innovation.

The lead-time on changes from data teams is high and in practice they cannot experiment. Much of this comes not so much from the complexity of the change they have to make but from the sheer size of the overall codebase.

### Archives And Forked Codebases

One implication of data changing over time and logic encoded within the code has been the necessity of archives, particularly the keeping alive of old codebases and even old operating systems probably with many security holes and which are difficult to manage. The archive problem is larger than this one issue, but the need to maintain old codebases and the security and admin implications of that rise solely from the decidion to keep business logic within the codebase.

### Being "At The Table" for Modelling Decisions

Currently, developers often need to be "at the table" for the modelling of the scientific data. This can take considerable developer time, either by attending meetings or by a back-and-forth as to what is practical. There is obviously a need for web design teams to influence the decision making in these cases, but it shouldn't be necessary for developers to be present to answer questions of "what's possible" as the tools avialable should be clear, and the questions pushed back to the scientists and designers.

## Impact on ZMenus

Most of this document concerns the motivation for the design of the genome browser as a whole, not specifically the zmenus. But the "top half" of zmenus are an important part of this goal. On their own, the visual parts of the genome browser don't really give sufficient implementation to "drop in" a track and it be at all useful. The summary part of a zmenu is an important to avoid some shape or track being mere "mystery meat".

In essence, though they are simple and small in code terms zmenus are essential in making a track at all useful: they are much more important in terms of experience. We've seen from tracks temporarily without them that the absence of zmenus make them of little use. From a user and data provider's perspecitve, zmenu summaries are a "good proportion" of the browser experience.

Effectively, without the ability to give at least some summary information and links about an item the genome browser degenerates into avery complicated way of drawing rectangles as it doesn't address the concerns in this document. Changes of the kind we want to make simple and clear become 50% simple and clear but then degenerate into the old "business as usual" when it comes to zmenus, requiring the slow addition of data to data-sources, it being modelled, and then new conditionals and cases added to the codebase.

For example, should a data provider add some new property to their data ("confiednce", or "quality", or "type", say) they would be able to represent this visually, with the advantages of the architecture described in this document, but then face the old challenges to have it appear in the zmenu.

Many of these challenges are "all or nothing" in the sense that almost their entire cost follows from needing to address them even partially.

## Goals

## Specific Motivating Examples

## Architectural Space

## Architectural Decision

## Structural Implications

## Interntal Shape

## Future Examples

## Current Status
