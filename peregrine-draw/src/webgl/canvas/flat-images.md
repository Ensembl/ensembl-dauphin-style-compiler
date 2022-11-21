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

A `CanvasInUseAllocator` allows you to create a `CanvasInUse`.
