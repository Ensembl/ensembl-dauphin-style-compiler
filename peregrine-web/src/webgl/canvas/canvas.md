# Flat Canvases

## Introduction

Flat canvases are 2d, pixel-based canvases. These can be used as textures for webgl objects. Flat canvases are used to render text. The reason for this decision is a bit complex. In the long run, flat canvases will be required for things like heat-maps. Text can be rendered on flat canvases relatively easily, so this was chosen as the initial use-case. It wouldn't surprise me if ultimately text moved off flat canvases to some other technique, but flat canavses themselves will likely always be needed.

The `Flat` type in flat.rs implements flat canvases themselves. This is relatively simple. The problems comewhen binding these to WebGL. It's necessary to create composite canvases as textures, as WebGL only supports a few textures in any run. This greatly complicates the rendering process.

## Packing

The flat canvases contain rectangular elements. These are arranged to minimise space lost. Rectangle-packing is an NP-hard problem, but there are many ways of approximating it. Sadly, many of these approximations are too slow to run in real-time. From among the broad classes of algorithms (shelf, guillotine, skyline, etc), only shelf algorithms have sub-O(n^2) statistics. These are rather poorly performing in terms of packing efficiency, but typically run in O(n log n) or O(n log^2 n). Most of these shelf algorithms are greatly assisted by pre-sorting of the rectangles before adding them. The impact of this on the broader architecture is that our drawing needs to be in two phases: one to collect all the space needs and the second, after sorting and then packing, to draw on those allocations.

Shelf-based algorithms reserve horizontal stripes of an image (shelves) for its allocations. Rectangles are then placed left-to-right in these shelves, typically top-aligned. The next shelf opens entirely beneath the previous one. We sort the entire stream into decreasing-height order to simplify allocation within shelves: once some rectangle has been placed, no further rectangles will need to make the shelf larger as all are less high. In theory area-based sorting has a slight advantage over height-based according to survey papers, but the difference tends to be marginal.

The result of shelf-based algorithms is an approximately triangular shape on each shelf. To reclaim some of this space (and to make up for our inability to rotate images), we make an optimisation. If more than a fixed fraction of a shelf is "wasted" at any point, a "sub-shelf" is opened beneath the first, of the available unused height. As we rely on any rectangle being placable in any shelf (due to the decreasing height ordering), some of these new sub-shelves need holding in a "waiting queue" until the objects being allocated are small enough that these shelves are candidates for use at which point they are removed from the holding queue and placed alongside the other shelves.

This algorithm is implemened in packer.rs with the method `allocate_areas`. This takes an array of sizes and returns a canvas size, and an array of origins 1-to-1 with the array of sizes. As such, it is isolated from the rest of the packer code.

See [alloc.md](alloc.md) for details of this algorithm.

## Binding

A second complication is that binding textures in WebGL is very slow. While we may only want one or two per drawing and only perhaps three drawings per screen, and a graphics card support at least 8 textures (and typically 32) this can vary greatly depending on what's to be drawn so managing the bindings effectively is both necessary and annoying. Added to this, the mess of there only being eight slots available per-drawing doesn't make matters any simpler.

These tasks are handled by the bindery. The role of the bindary is to create and delete textures and cycle through them according to graphics card capacity to minimise the number of times this has to be done, even across drawings. The main class of thebindery is `TextureBindery`. This is a globally-scoped object. A particular process passes a `FlatId` to the `allocate()` method and gets a series of operations to perform in a `Rebind` object. Once executed, the `gl_index()` method will point to the index of the flat. `clear()` is run for each process and is a bit of a misnomer. It only _allows_ the bindery to reallocate any objects up to this point, should it be pressing to do so. `free()` creates a rebind object which when run will delete texture (during discarding).

`Rebind` is a separate method so that an entire `WebGlGlobal` object may be passed to it. `TextureBindery` is itself in `WebGlGlobal`, so to do the operations internally would create a borrowing nightmare.

The `WebGLTexture`s generated are stored in the `TextureStore` object, also part of the global state. No operations on this store are visible outside the `Rebind` object.

No attempt is made to preserve binding index between each run as this can be changed efficiently. Each `clear()` resets the index, so textures which stay in the cache may well change their index for different drawings.

## Pipeline

The core pipeline for canvases is in three stages

* `Weave` is an enum describing various properties to be set when attaching the texture in WebGL. These properties all influence the visual appearance of the texture (eg bluring) and so different objects will want to be drawn on canvases with different "weave".

* `FlatPlotAllocator` in flatplotallocator.rs allows consumers to request a set of regions of the given sizes and returns a `FlatPlotRequestHandle` representing that request.

* `DrawingFlatsDrawable` in drawingflats.rs is generated from `FlatPlotAllocator` when all requests have been added by its `make` method. This has methods for retrieving the origins and CanvasId for the generated canvas.

* `DrawingFlats`, also in drawingflats.rs, represents all the flat canvases for a drawing when the canvases are complete. The `built` method on `DrawingFlatsDrawable` makes this object. The flats can be added to a process (via `add_process`) and also discarded.

* `FlatStore` xcontains the means to access the Flat canvas itself.
No other objects directly contain flats, only `FlatId`s. These `FlatIds`can then be used with `FlatStore` to look up the flat itself.

```
+----------------------+ <----- Request regions
| FlatPlotAllocator    | -----> Get FlatPlotRequestHandle
+----------------------+
       | make
       v
+----------------------+ <----- Supply FlatPlotRequestHandle 
| DrawingFlatsDrawable | -----> Get origin and canvas of allocation
+----------------------+        (which you may now draw on)
       | built
       v
+----------------------+
| DrawingFlats         | ----> Can be added to processes for binding
+----------------------+
```

This pipeline is run from the methods of the `DrawingBuilder` class which are called in turn from `GLCarriage`'s constructor.

## FlatStore

FlatStore's role is to take a FlatId and supply the corresponding Flat. It is necessarily global across the whole context. It can also supply scratch canvases for doing things like measuring. FlatStore is separate to avoid various borrowing headaches.

Its has an unremarkable, CRUD-like API.

Of the externally visible methods:

* `new` creates the flatstore.
* `scratch` returns a cnavas of at least thegiven size.
* `allocate` allocates a new flat returning the FlatId.
* `get` retrieves a given flat.
* `discard` disposes of a given flat.
* `discard_all` disposesof all flats.

## FlatPlotAllocator

FlatPlotAlloctor is tasked with collecting requests for allocations and then, on its final call, allocating them and passing the result to the newly-created DeawingFlatsDrawable.

Each request can be for multiple regions. This ensures a single request, for example for a mask and texture, end up on the same canvas and share accessors for such in the combined request.

Its public API is

* `new` creates the FlatPlotAllocator.
* `allocate` requests the given allosations and returans a `FlatPlotRequestHandle`.
* `make` allocates the regions and compiles that invormation into a newly-created DrawingFlatsDrawble, which it returns.

Internally, all operatiosn are segregated by weave as each weave needs its own flat. When make is called, a HashMap of weave_allocators is created for each weave type mentioned. Each request is added to the relevant `WeaveAllocator` and then `allocate` called on each. For each canvas a canvas is allocated via the newly-created `DrawingFlatsDrawable`'s `make_canvas` method. This allows the use of the canvas on this drawing to be registered.

Once each has been allocated, the requests are again iterated through and the `origins` method is called for each id. Finally the `DraingFlatsDrawable` has its `add` method called to register the given request and allocation position.

The individual `WeaveAllocator` just accumulates requests on adding in objects of type `WeaveALlocatorData`. Once `allocate` is called, arrays of sizes requested are built and passed to `allocate_areas` (the main packing algorithm). When the origins are returned, their values are added to the relevant `WeaveAllocatorData`. The `origins` method uses the `WeaveAllocatorData` to access these origins on demand.

## DrawingFlatsDrawable

`DrawingFlatsDrawable` represents the intermediate stage of drawing on flat canvases, after allocations have been made but before the drawings have been finalised. At this stage the size and the locations of the allocations must be added to the WebGL side ofthings as this information is not preserved into `DrawingFlats`. In this stage drawers draw on the canvas. There is an internal API for `FlatPlotAllocator` comprising

* `new` to dreate the object.
* `add` to add a handle on the given canvas at the given origin.
* `make_canvas` to request a new canvas.

These are not externally visible and are all invoked within the `make`  method of `FlatPlotAllocator`.

Unlike the "record and later do" architecture of `FlatPlotAllocator`, `DrawingFlatsDrawable` creates an empty third pipeline stage object, a `DrawingFlats` on creation and calls any necessary methods as it goes along.

The public API is:

* `origins`: the origins associated with the given `FlatPlotRequestHandle`.
* `canvas`: the canvas associated with the given `FlatPlotRequestHandle`.
* `built`: when all is done, this method returns the (previously embedded `DrawingFlats` object).

The general progress of drawing is that `origins()` and `canvs()` for the known `FlatPlotRequestHandle` are called, that canvas retrieved from the `CanvasStore` and then various drawing operations performed upon it.

## DrawingFlats

`DrawingFlats` knows only about the (whole) flats used by this drawing and is unaware of anything to do with allocations or drawing (this being over). It is preserved in the drawing after the creation process is complete and is crated within `DrawingFlatsDrawable` and exposed via `built()` when the drawing is complete.

The public API is

* `add_process` is called at rendering time to add all the canvases for a drwaing to be drawn.
* `discard` is called when a drawing is deleted.
