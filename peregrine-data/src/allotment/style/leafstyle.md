# LeafStyle

THis directory contains the basic style omdel for containers and leafs. In this context when we use the word "Leaf" we are talking about the objects which ultimately contain the programatic equivalent of the settings from the style scripts.

These are mainly a wide range of style objects used during the styletree building process (including the styles of containers, the very existence of which doesn't escape from the styletree process).

## Objects here

`InheritableStyle` -- the set of properties which can be specified in containers and trickle down into leafs (where they take effect).

`UninheritableStyle` -- the set of properties which can only be specified on a leaf itself.

`ContainerStyle` -- the set of properties which configure containers themselves.

`LeafStyle` -- this is the ultimate style of a shape, so it may be a surprise to see it here, early in the building process, before Floating and Anchoring. The secret is thatthe LeafStyle is generated at this stage and hidden inside those objects until the rest of their contents are not needed and the LeafStyle is revealed!
