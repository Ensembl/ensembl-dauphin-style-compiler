# Flat Images

## Introduction

Flat images are used as textures in the genome browser to allow pixel-based data such as text
and bitmap images. These can be very large (due mainly to the quantity of text), and so efficiency is potentially a concern.

There are three essentially independent stages which can be potentially slow either directly or through fragmentation:

1. creating the canvas element itself;
2. drawing on the canvas element;
3. transferring the canvas element from the CPU to GPU.

## The Canvas Element itself

The `CanvasSource` is a source of `HtmlCanvasElement` objects. It incorporates a cache of unused elements to reduce the number ofelements created. The main speed saving here is in reducing memory fragmentation over time.

## The canvas in use

When a canvas is needed we must take out a context which is stateful. The operations on the canvas are also not idiomatic for our use-case. To manage these two aspects, a `CanvasAndContext` object wraps an `HtmlCanvasElement` and provides better primitives for our use. This object is not clone, so a `CanvasInUse` object wraps it and provides mutability.

At this stage, no WebGL use has entered our API. We could equally use a `CanvasInUse` for other purposes (but don't, at the moment).

A `ScratchCanvasAllocator` allows you to get a direct, temporary lease on a `ScratchCanvasAllocator` which can be much faster, but you can't guarantee who else might draw on it, so it's used for quick borrow/use/dispose for exmaple, when measuring texts.

## Tessellating

The things we need to draw are small; so small that they can't have a canvas each but must be packed together into a single canvas.

We call the narrow packing of blocks "tessellation," and the broader process of getting sizes, packing, drawing, etc the _composition_ of _canvas items_.

The individual canvas items (text, images, etc), are of different types but are unified in implementing the `CanvasItem` trait. One canvas is composed with items by a `CompositionBuilder`. There are multiple "types" of canvas, even for one drawing (typically three or four), as they differ in various settings like wrapping behaviour, blurring, and the way they must be packed (in 2d, horizontally, verticall, etc) etc. Each type is a different `CanvasType`. The `DrawingToolsBuilder` struct maintains one `CompositionBuilder` per `CanvasType`.

The details of this process are described in a later section, however the general scheme is that items are added to a `CompositionBuilder` and then a build method called which yields a `CanvasInUse`.

## Binding to GPU

Eventually a `CanvasInUse` which has been prepared as a texture by passing through `CompositionBuilder` will need to get to the GPU and bound for use in WebGL as a texture. GPUs have varying maximum limits for the number of textures, and the texture process is performance critical, so we need to do this sensibly.

In practice there are an small number of integer-indexed slots on the GPU, and we must bind each canvas to one of them.

Fortunately, there are generally more slots than canvases we need for any one program, so we can try to keep old textures "hanging around" for next time we draw. Unfortunately, there are not so many slots that the issue can be ignored entirely: sometimes canvases will have to be evicted from slots and recreated later, if and when needed.

This process is known as binding or activation. Externally the API is simple. Each `CanvasInUse` has an `activate()` method which binds the canvas to a slot if not currently bound. This method takes a `TextureBinding` singleton object which manages the process. The method returns an integer, the slot, to be put into a WebGL uniform as a handle. `TextureBinding` also includes a `clear()` method which globally marks canvases as not currently in use, run at the start of a program.

Behind the scenes, the algorithm is complex. It is split into two parts. `Binding` contains the core cache algorithm and the source details its operation in comments. `TextureBinding` layers on WebGL specifics. Binding is highly polymorphic to allow testing of the algorithm independent of WebGL.




## Details of tessellation

The stages are as follows.

1. During the prepare phase of iterating through the GLShapes, the code notices that a particular code needs a texture for support so it retrieves the relevant `CompositionBuilder`, creates a `CanvasItem` and adds it to the `CompositionBuilder`. In response, it gets a `CanvasItemAreaSource` which will later be queryable for the coordinates on the texture. It keeps this object safely with inside shape.

2. At the end of the prepare phase, the tools builder prepare step is called. This, in turn calls `draw_on_bitmap` on each of its `CompositionBuilders`. Inside which ...

3. For each ``CanvasItem`` added to the `CompositionBuilder`, it calls the `calc_size` method, to ask the item how much space it should reserve in the canvas.

4. The tessellation algorithm is run to pack these size requirements. It also returns the overall size and `draw_on_bitmap` creates a bitmap of the relevant size.

5. Through the magic of shared references all of the `CanvasItemAreaSource` objects scattered to the four winds are set to return the location of the allocated bit of the canvas determiend in tessellation should they be asked.

6. The `draw_on_bitmap` method is called on every `CanvasItem` to allow the contents to be drawn on the new canvas.

7. The `draw_on_bitmap` method of `CompositionBuilder` finally returns with the allocated canvas object.

8. The prepare step registers the canvas for this drawing in `DrawingCanvasesBuilder` to ultimately become part of the `Drawing` object, keeping a handle on the canvases and allowing them to be mapped as textures.

9. The prepare step of the tools builder finally finishes. Now it's the draw phase of iterating through GLShapes. The code for adding WebGL co-ordinates will notice the `CanvasItemAreaSource` for the object and use it to get the co-ordinates of the canvas item and add it to the WebGL data.

Eventually the `CanvasInUse` structs made by the `CompositionBuilder` end up in the `Drawing` struct for safe-keeping. When the drawing is dropped, the `Drop` trait on the lease ensures the canvas get returned to the cache.

The tessellation algorithm used is a custom, shelf-based allocator which compromises on speed and packing density. It is inside `packer.rs` which exposes `allocate_horizontal`, `allocate_vertical` and `allocate_areas` to the rest of the code.

