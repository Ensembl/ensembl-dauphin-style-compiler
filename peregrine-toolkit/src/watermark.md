# Introduction

The watermark datastructure maintains a "height" of various ranges along a single axis. This height is initially
zero, but can be "raised" by some amount within a range. It starts at the heighest existing point in that range
and then is increased by some amount.

For example an empty watermark could have an element of height 3 added from 5 to 12. That range is now at height 3. A further range of height 2 could be added between 2 and 6. This region will now have height *five* because part of that region (between five and six) was already at heigh 2 and we are adding 3 more.

```
       2      5  6               12
              [3333333333333333333] _ height 3
       [222222222]                  _ height 5
    
{  0  }{    5    }{        3      } <- stored watermark values
```

It's easy to tell how this could sit at the core of a bumping algorithm!

The watermark data-structure is represented by a start offset and a height. The height continues until some other height begins. For example, in the above:

```
(0,0), (2,5), (6,3), (12,0)
```

It is initialised as `(0,0)`.

# Algorithm

The algorithm to insert a range is as follows:

```
insert(our_start,our_end,out_height):
  # Part A
  # these inital steps can be a single linear scan
  height_to_left := find_last_height_before_start()
  dumped_entries := remove_all_existing_entries_in_our_range()
  right_position := find_first_entry_at_or_after_end()
  height_to_right := find_first_height_at_or_after_end()

  # Part B
  if length(dumped_entries) > 0:
    final_height := max_height(dumped_entries)
  else:
    final_height := height_to_left
  new_height := prev_height + our_height

  # Part C
  if new_height != height_to_left:
    insert(our_start,new_height)

  # Part D
  if right_position != our_end:
    insert(our_end,final_height)
```

We are careful to merge any equal heights as we go along as this is likely to have a big performance impact by minimising the data-structure size. As our limiting steps relate to search this will keep the algorithm fast.

### Part A

In Part A we do a linear scan starting at a given index point for varoius pieces of information used later. Keeping this step efficient is what determines the low-level data-structure design. As we traverse, we remove any entries "under" our allocation.

### Part B

We determine our height by looking at the dumped pieces to find the highest and adding our own height.

### Part C

This considers the height to the left. If it matches our height then we need not add a new entry. Otherwise we need to add a new entry.

### Part D

Now we consider the right. We need, directly at the end to the end of the range, to re-establish the last value. If a new height is established directly at our end already, we need not insert a new entry. Otherwise an entry needs to be inserted with the prev_value, since it "pokes out" of our allocation.

## Data Structure

The algorithm requires a data-structure with operations which are supported naturally by a B+ tree.

# Smoke test diagram

Add 5-12, height 3 -> 0
```
012345678901234
     [==3===]
```
Add 2-6, height 2 -> 3
```
012345678901234
     [==3===]
  [=2=]
```
Add 6-8, height 2 -> 3
```
012345678901234
     [==3===]
  [=2=|2]
```
Add 0-1, height 2 -> 0
```
012345678901234
2]   [==3===]
  [=2=|2]
```
Add 9-14 height 1 -> 3
```
012345678901234
2]   [==3===]
  [=2=|2][=1==]
```
Add 7-13, height 1 -> 5
```
012345678901234
2]   [==3===]
  [=2=|2][=1==]
       [==1==]
```
Add 0-9, height 4 -> 6
```
012345678901234
2]   [==3===]
  [=2=|2][=1==]
       [==1==]
[===4====]
```
