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

We call the narrow packing of blocks "tessellation," and the broader process of getting sizes, packing, drawing, etc the _composition_ of _canvas items_.

The individual canvas items (text, images, etc), are of different types but are unified in implementing the `CanvasItem` trait. One canvas is composed with items by a `CompositionBuilder`. There are multiple "types" of canvas, even for one drawing (typically three or four), as they differ in various settings like wrapping behaviour, blurring, and the way they must be packed (in 2d, horizontally, verticall, etc) etc. Each type is a different `CanvasType`. The `DrawingToolsBuilder` struct maintains one `CompositionBuilder` per `CanvasType`.

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

## Binding to GPU

Eventually a `CanvasInUse` which has been prepared as a texture by passing through `CompositionBuilder` will need to get to the GPU and bound for use in WebGL. GPUs have varying limits for these things and the process is performance critical, so we need to do this sensibly. In practice there are an small number of slots on the GPU and we must bind the canvas to one of them.

Fortunately, there are generally more slots than canvases we need for any one program, so we can hopefully keep old textures "hanging around" for next time we draw. But unfortunately, there are not so many that the issue can be ignored entirely: sometimes canvases will have to be evicted.

Essentially, a canvas which we might wish to bind can be in one of three states:

1. It could be *UNBOUND* and the GPU generally unaware of it. It would still be a `CanvasInUse` and have a drawing on it which cannot be stolen for other uses, but not used in the WebGL program running at that time.

2. It could be *ACTIVE* and bound to the GPU and in use for the current program.

3. It could be *VESTIGIAL* and still bound from an earlier operation, hopefully staying so until it is next needed, saving an expensive operation.

There are three operations which can be performed on these states:

1. `allocate()` is a request to bind acnavas to a slot. Any empty slot can be used, as can any binding a vestigial canvas (choosing the least-recently used first).  If a slot used by a vestigial canvas is used it becomes unbound. The allocated canvas becomes activ.

2. `clear()` is called at the start of a program run: all active canavses are relegated to vestigial.

3. `free()` is called on dropping an individual canvas, making it unbound.

Binding is primarily managed by tokens internally within `CanvasInUse`. In a field inside `CanvasInUse` is a `SlotToken`. When a canvas is used (the allocate procedure) a flag is set on the canvas indicating the canvas is active. When the `SlotToken` is dropped, the `free()` procedure is run. `clear()` is run centreally in `Binding` alongside the `allocate()` procedure. This is acceptable in our Drop-based-freeing strategy as `clear()` is actually an initialisation operation.

