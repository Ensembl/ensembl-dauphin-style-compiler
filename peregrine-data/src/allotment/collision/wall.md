# Wall

## Introduction

The wall algorithm is designed for lovelier bumping than the default algorithm when we can assume that the elements to be bumped are all (near enough) the same height. This is a much simpler case than the general bumping algorithm, so we can do a better job at getting things neat. In this case we can put things into "rows" like lines of bricks in a wall.

In general (such as when used with genes) the elements to be bumped can differ greatly in size, it's more like a dry-stone field wall than a brick wall, and so the amount of effort needed to place things appropriately is greatly increased and the result less satisfactory, espeically given the terra incognita of whatever the size may be of the elements to the left and right of the current display.

So if your elements can all be guaranteed to be roughly the same height, use "wall" instead of "bumper". The height chosen will be the largest element you specify for *all* of the elements in the wall, so don't have any tall outliers!

Wall collects data in the same way as bumper, which is described in the first section of `collision.md`, but then applies its own algorithm to the positioning.

## Algorithm

The wall algorithm allocates "courses" of bricks known as rows. Each row has a height. If the height changes in incremental update, incremental update is abandoned.

Each row has a range within which it is occupied. When a new request comes in, rows are searched from the least to the greatest to find one where the request will fit, and the occupied range for that row updated.

That's it. It's orders of magnitude simpler than the bumper algorithm!