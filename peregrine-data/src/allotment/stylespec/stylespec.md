# StyleSpec

The stylespec is a box model a little like CSS which is designed to lay out boxes in the genome browser. Box models are slow and inefficient to calculate and are often the limiting factor in the number of objects to display. The points we vary from the general scheme of CSS are due to:

1. our unusual, multiple, overlaid coordinate systems of one-dimensional stetch;

2. to maintain efficiency with thousands of objects.

## Overview

Style boxes are maintained in a tree. They are of one of two types, containers or leafs. A container can contain leafs and other containers, but cannot be drawn in. A leaf cannot contain anything, but can be drawn inside.

Most of the styling for a leaf can be specified within one of its ancestor containers (closest wins) or within the leaf itself. Container properties are not inherited and apply only to the container itself.