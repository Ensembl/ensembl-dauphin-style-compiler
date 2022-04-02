/* Thealgorithm proceedsin an ever-expanding front from left to right or right to left. A "castle" represents 
* the current "front". In other words, it represents a shape something like these three examples. In these
* images allocation has proceeded a certaindistance from left to right and the castle show the boundary of
* allocated space.
* 
* -------+             ----+     -------+
*        |                 |            |
*        +-------+         |            |
*                |         |            +----+
*    +-----------+         |                 |
*    |                     |                 |
* ---+                 ----+     ------------+
* 
* The castle data strucutre is, in turn, used to update the CastleCursor data-structure. The CastleCursor is
* a "cut" through the castle at some horizontal position, and represents a set of "occupied" vertical ranges
* 
*      |
* -----|-+                      1
*      | |                      1
*      | +-------+              1
*      |         |              1
*    +-|---------+              1
*    | |                        0
* ---+ |                        0
*      | <- a castle cursor ->  0
* 
* A CastleCursor moves only in the direction of the castle (left-to-right in the above castles). To allow
* update of a CastleCursor, the ends of each of the vertical regions is stored in a BinaryHeap within the
* Castle. To advance a CastleCursor from a to b, all end-points from a to b are recalled sequentially and
* removed from the cursor. When an allocation is made, the allocation is added to both the castle cursor and
* the castle indepdenently.
* 
* The castle-cursor is used to find the least value with a given headroom. There are typically only a few
* entries in the caslte cursor, such that a simple linear scan probably wins out as a representation.
*/

use std::collections::BinaryHeap;

pub struct CastleCursor {
    contents: Vec<(i64,i64)>
}

impl CastleCursor {
    fn new() -> CastleCursor {
        CastleCursor {
            contents: vec![]
        }
    }

    fn maybe_merge(&mut self, index: usize) {
        if index+1 < self.contents.len() {
            if self.contents[index+1].0 == self.contents[index].0 + self.contents[index].1 {
                /* bridges, remove second entry */
                self.contents[index].1 += self.contents[index+1].1;
                self.contents.remove(index+1);
            }
        }
    }

    /* We can assume that we never get overlapping regions as Castle never produces them, that means
     * offset never equals the offset of an existing item.
     */
    fn in_use(&mut self, offset: i64, height: i64) {
        let mut insert_before = self.contents.len();
        let mut maybe_bridge = None;
        for (i,(item_offset,item_height)) in self.contents.iter_mut().enumerate() {
            if *item_offset+*item_height == offset {
                /* abuts to the left (and maybe right), modify length to cover us */
                *item_height += height;
                maybe_bridge = Some(i);
                break;
            }
            if offset+height == *item_offset {
                /* abuts to right (but not left), modify start and height and we are done */
                *item_offset -= height;
                *item_height += height;
                return;
            }
            if offset < *item_offset {
                insert_before = i;
                break;
            }
        }
        if let Some(index) = maybe_bridge {
            /* because maybe_bridge was set, abuts to the left (and maybe right). We're now out of the loop,
             * so we can check the next member. If it matches up, remove it.
             */
            self.maybe_merge(index);
        } else {
            /* Because maybe_bridge wasn't set, we abut neither left nor right, so need to insert.
             */
            self.contents.insert(insert_before,(offset,height));
        }
    }

    /* We can assume that a freed region is in use, because Castle assures that. Therefore our offset
     * will always be "within" some single entry. We may be able to delete the whole entry, modify it or,
     * in the worst case, have to split
     */
    fn free(&mut self, offset: i64, height: i64) {
        let mut maybe_merge = None;
        let mut insert = None;
        for (i,(item_offset,item_height)) in self.contents.iter_mut().enumerate() {
            if offset == *item_offset {
                /* starts at the start of the region (and maybe ends at the end) */
                *item_height -= height;
                *item_offset += height;
                maybe_merge = Some(i);
                break;
            }
            if offset+height == *item_offset+*item_height {
                /* ends at the end of theregion, but doesn't start at the start */
                *item_height -= height;
                return;
            }
            if offset > *item_offset && offset < *item_offset+*item_height {
                /* middle of a region, need to split */
                insert = Some((i+1,*item_offset+*item_height));
                *item_height = offset - *item_offset;
                break;
            }
        }
        if let Some(index) = maybe_merge {
            self.maybe_merge(index);
        } else if let Some((index,old_end)) = insert {
            self.contents.insert(index,(offset+height,old_end-(offset+height)));
        }
    }

    fn find(&self, min_height: i64) -> i64 {
        let mut attempt = 0;
        for (index_offset,index_height) in self.contents.iter() {
            if index_offset - attempt <= min_height {
                return *index_offset - attempt;
            } else {
                attempt = index_offset + index_height;
            }
        }
        attempt
    }
}

/* Note when decreasing end is less than start. The cursor is advanced to the GREATEST coordinate and then
 * the other end, with the lesser value, is set as end.
 */
pub(super) struct Castle {
    cursor: CastleCursor,
    regions: BinaryHeap<(i64,i64,i64)>, // (end,offset,height) // increasing negates end as maxheap,
    maximum: i64,
    increasing: bool
}

impl Castle {
    pub(super) fn new(increasing: bool) -> Castle {
        Castle {
            cursor: CastleCursor::new(),
            regions: BinaryHeap::new(),
            maximum: 0,
            increasing
        }
    }

    pub(super) fn advance_to(&mut self, position: u64) {
        while let Some((stored_end,offset,height)) = self.regions.peek().cloned() {
            let end = if self.increasing { -stored_end } else { stored_end };
            if end > position as i64 { break; }
            self.regions.pop();
            self.cursor.free(offset,height);
        }
    }

    pub(super) fn allocate_at(&mut self, end: u64, offset: i64, height: i64) {
        let stored_end = if self.increasing { -(end as i64) } else { end as i64 };
        self.regions.push((stored_end,offset,height));
        self.cursor.in_use(offset,height);
        self.maximum = self.maximum.max(offset+height);
    }

    pub(super) fn allocate(&mut self, end: u64, height: i64) -> i64 {
        let offset = self.cursor.find(height);
        self.allocate_at(end,offset,height);
        self.maximum = self.maximum.max(offset+height);
        offset
    }

    pub(super) fn maximum(&self) -> i64 { self.maximum }
}
