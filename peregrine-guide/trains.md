# Trains

## Introduction

Trains are responsible for managing and grouping the displayed genome browser panels. More specifically, it is responsible for:

* determining which panels should be prepared and drawn in response to position and scale input;
* scheduling requests for new panels at new positions and scales;
* managing the transitions between sets of panels;
* interacting with the drawing crate;
* disposing of unwanted panels.

An individual panel is known as a `Carriage`, representing a certain region of a certain chromosome at a certain scale. Carriages are assembled into `Train`s. A train represents a certain chromosome at a certain scale. At any moment in time it may be composed of a small number of carriages representing the reginon being displayed (or near-displayed). As a user pans carriages are added at one end of a train and removed from the other. A `TrainSet` is a singleton representing the trains currently of interest.

While `Trains` are dynamic in terms of the carriages which they can contain, a `CarriageSet` only represents a single set of such carriages. So for a given train there is at any time one active `CarriageSet` and in order to add or remove carriages to a train(for example due to panning) a new `CarriageSet` is created and replaces the old. An efficient constructor allows a new `CarriageSet` to be composed from one which it partially overlaps without copying individual `Carriages`.

The train modules interface asynchronously with much of the rest of the `peregrine-data` module but also have extensive shared state and potential reenterancy. This could create a nightmare for locking. To avoid this, a `CarriageEvent` queue object is created for each top-level request which collects requested actions from the locked state. When this is done andthe locks released, the queue is emptied and those actions performed.

## Train Types

A trainset contains up to three trains at any one time.

* `current` is the trainset being displayed to the user. If we are in the middle of an animation between trains, it is the train fading _out_.
* `future` only exists during a fade. It represents the train fading in. As soon as the fade is complete, this train vacates future and becomes current.
* `wanted` is present if the user has requested some other region or scale but the data is not yet available. Once `wanted` is ready it is transferred to `future` and the transition animation begins.

The general order is wanted -> future-> current.

If a user interacts in such away that the train which is wanted changes before it has time to load it is simply immediately replaced. However, future is never interrupted in its transition to current to avoid flicker.

The `quiescent` stick is the stick which we are currently aiming to display and so where effort should be spent on updating. If present this is wanted, if not furutre and if that is also missing current.

## TrainSet

Interaction with the `TrainSet` from outside the module is by way of two calls.

`set` updates the position of the browser via a `Viewport` object. `transition_complete` is a callback from the drawing code indicating an animation between trains is complete.

### set() and transition_complete()

To set position, the `TrainSet` creates a `CarriageEvents`, calls `set()` on a locked `TrainSetState` and then exectues those events.

First `maybe_new_wanted` is called with the updated viewport to see if the region or zoom level have changed sufficiently that we need to put a new train in `wanted`. If so, the old wanted is simply replaced. If a new train is constructed, a "Train" event is added to the event queue.

Then each train on the correct stick is informed of the new position via its `set_position` method, so that it can update its carriage list, if necessary. The train does this through updating its CarriageSet. The only potential event added by this operation is a "Carriage" event.

Finally `promote` is called which managers animations and so trains getting from `wanted` into `future`. The train has a predicate `ready()`. When this is true and there is no train currently in future (ie no animation in progress), the train is moved from wanted into future. At this point the train is set as active with `set_active()`. When a train is newly active, the "Transition" CarriageEvent is added to tell the CarriageEvents.

When the transition is complete, the drawing code calls the `transition_complete()` method. When this method is called, the old `current` is disposed of via `set_inactive()` and future is used to replace current. Finally `promote` is again called, in case the freeing of future allows a wanted train to begin to transition.

### Loading Carriages

The "Carriage" event added in `set_position` leads, when the queue is run, to a call to `run_load_carriages()` in `TrainSet` for all the wanted carriages. This creates a new thread.

First this thread calls `load_carriages()` which calls `load()` on each carriage in parallel and waits for it to complete. When the carriages are loaded `update_trains()` is called. `maybe_ready()` is called on each train, which updates its ready flag. Again `promote()` is now called to perhaps shift wanted into future. For each of the active trains `set_carriages()` is called. If the carriages are now all ready, a `Set` event is created to update the UI of the new carriage list.

### CarriageEvents

There are the following event types

* `Train` -- created when a new train is created to get metadata about the train (for example stick length). Called in the constructor of a `Train`.
* `Carriage` -- created when a new CarriageSet includes carriages not in the previous set to create a new carriage. Ultimately it is this event which will cause the main data loads.
* `Set` -- sets the current carriages to display when they are updated by calling the `set_carriages()` method of the drawing crate.
* `Transition` -- starts the train transition in the drawing crate.
