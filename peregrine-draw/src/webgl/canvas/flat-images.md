# Flat Images

## Introduction

Flat images are used as textures in the genome browser to allow pixel-based data such as text
and bitmap images. These can be very large (due mainly to the quantity of text), and so efficiency is potentially a concern.

There are three essentially independent stages which can be potentially slow either directly or through fragmentation:

1. creating the canvas element itself;
2. drawing on the canvas element;
3. transferring the canvas element from the CPU to GPU.

## The Canvas Element itself

The web_sys HTML canvas element is wrapped inside a `Canvas` object purely to make querying its size more efficient than doing so through the browser API. In all other ways it should be considered a raw HTML canvas DOM element.

The `CanvasSource` is a source of these `Canvas` objects. It incorporates a cache of unused elements to reduce the number ofelements created. The main speed saving here is in reducing memory fragmentation over time.

## The canvas in use

When a canvas is needed we must take out a context which is stateful. The operations on the canvas are also not idiomatic for our use-case. To manage these two aspects, a `CanvasAndContext` object wraps a `Canvas` and provides better primitives for our use. This object is not clone, so a `CanvasInUse` object wraps it and provides mutability.

At this stage, no WebGL use has entered our API. We could equally use a `CanvasInUse` for other purposes (but don't, at the moment).

A `ScratchCanvasAllocator` allows you to get a direct, temporary lease on a `ScratchCanvasAllocator` which can be much faster, but you can't guarantee who else might draw on it, so it's used for quick borrow/use/dispose for exmaple, when measuring texts.

## Tessellating

The things we need to draw are small; so small that they can't have a canvas each but must be packed together into a single canvas.

The algorithm used is a custom, shelf-based allocator which compromises on speed and packing density. It is inside `packer.rs` which exposes `allocate_horizontal`, `allocate_vertical` and `allocate_areas` to the rest of the code. These functions map a list of sizes to a list of offsets to non-overlapping rectangles. The three methods exist because the algotihm needs to bedifferent depending on the desirededge-behaviour of the canvas (wrapping or not) and these three are unified in `CanvasWeave::tessellate` which calls the appropriate method.