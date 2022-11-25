# Gesture

## Overview

The gesture code handles the algorithm to translate between various physical mouse and finger events (mouse down, mouse move, pinch, etc) into conceptual-level messages about the action that was performed: click, drag, marquee drag, pinch, etc.

The gesture code doesn't concern itself with the preceeding stages of sourcing the events from the browser. Nor does it concern itself with the downstreamconsequences of the actions (zoom in, pan left, etc), just the stage in between.

The sole exception to this is that the gesture code *does* delegate the mapping from what happened (which it has just painstakingly determined) to the action to perform as a consequence, to an algorithm elsewhere in the codebase, as the data is "on the way out" of the module. This is just to keep the gnarly datatypes it needs private, and shouldn't be considered a core function of this code.

The gesture code is responsible for the spectres which aid with performing actions (eg the marching ants) and (largely) for managing the cursor shape.

```
[elsewhere] ----------> gesture code -----------------> [mapping]  ----------->
            mouse up                  draged left                  zoom in
            touch event               pinched                      move left
            ...                       ...                          ...
            mouse move                dragged out area             move down
```

This sounds like a simpletask but it isn't. There isprecious little information in the physical mouse events to distinguish between different actions, and an awful lot of meaning is layered onto a very few events. As this code is likely to change, some effort has been put into making a little walled-garden to allow easy examination and changing of this code, known as the gesture state machine. The code to run the gesture state machine is in `core` and the machine itself in `node`. You will see in `node` a file for each kind of action.

## External API

`Gesture` is the entry point to this code. It provides three methods: `new` for when the mouse first goes down; `drag_continue` for when an action is ongoing with themouse down; and `drag_finish` for when it is done. It takes two locations for a primarey and optional secondary touch point (this being used for example in pinching).

The output is a stream of `InputEvent` messages sent to the lov-levelinput manager (where they will be co-mingled with keyboard messages, etc, of the same general kind).

The touch points are known consistently as "fingers" as touch-screen is the most complex case. A mouse click istreated as a single finger in some position.

## The Gesture State Machine

Throughout the period between mouse down and mouse up the gesture state machine is in a single node state at any one time. For example, the `Drag` node state if the user is dragging, or `Unknown` if we are unceratin because the mouse has just gone down.

When in a state the node will receive messages concerning movement of the mouse which it may react to in a number of ways:

* it may send InputEvent messages to the rest of the genome browser;
* it may set the cursor shape;
* it may tell the driver that it should transition to a new node;
* it may create and manipulate spectres;
* it may set up timers for the node code to be called back to perform some of these actions after some delay.

The calls into the nodes (inside `GestureNodeImpl`) are:

* `new` -- called when a node is newly created (the node may wish to set the cursor or set timers);
* `timeout` -- called when a timeout expires
* `continues` -- the mouse has moved
* `finished` -- the action is over as the mouse has been unclicked.

Note that timeout callbacks are only run if there have been no transitions to a new node since they were set, toherwise they are discarded. This means there's no need to worry about cancellation.

The calls recieve an object called `GestureNodeState` which contains various useful objects for use by the nodes; `GestureNodeTransition` which collects actions which the node asks to be performed; and various arguments specific to the function concerned. A common additional argument is `OneOrTwoFingers` which contains the current location of the start and current finger positions.

## Inside the state machine runner

You shouldn't need to look inside the runner too often, but you may need to tweak it from time to time to bring in additional data.

The core code is in `gesture.rs` and comprises the main `Gesture` object and `GestureState`, its mutable internal state. `GestureNodeState` mentioned earlier is also defined here.

`finger.rs` contains all the irritating variety of types neede to represent the state of the current touch such as `OneFinger`, `OneOrTwoFingers`, etc.

`transition.rs` is the home of `GestureNodeTransition` mentioned earlier. This object collects requests from the nodes but only services them once the function is done and all the various locks are released (it runs them via the `apply` function). The code for managing `TimerHandles` is also here as it is closely tied to `GestureNodeTransition`. When you create a timer you get one of these opaque handle objects so that when the `timeout` call runs, the node can distinguish the sender.

`gesturenode.rs` is glue,bringing together the different node types in a single place.

`cursor.rs` manages the current cursor shape via handles. Though this sits inside the gesture code and is widely used by the same, it is also used elsewhere in the codebase (for examplein the hotspot code).
