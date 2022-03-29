/* A Watermark is the core data-structure of bumping. It maintains a piecewise-continuous maximum value along
 * a discrete dimension (i64). Initially this maximum is zero everywhere but pieces can be added to it. A piece
 * comprises a range along the dimension and a height. The height of the waterline is set so that the height in
 * the range supplied is set to the maximum existing value in that range plus the height given. The watermark 
 * also keeps track ofthe maximum value used anywhere. Because of the nature of bumping, these maximums are
 * usually best imagined being in the "down" direction. For exmaple watermark might do:
 * 
 *   0       3        6        10   11
 * 0---------+                       +-------
 * 1         |                  +----+
 * 2         +--------+         |                <-- BEFORE
 * 3                  |         |
 * 4                  +---------+
 * 
 * Add (5-8) height 2
 * 
 *   0       3     5      8    10   11
 * 0---------+                       +-------
 * 1         |                  +----+
 * 2         +-----+            |                <-- AFTER
 * 3               |            |
 * 4               |      +-----+
 * 5               |      |
 * 6               +------+
 * 
 * Internally a range is known as a "node" and is stored at its leftmost position. For example in the above
 * example, BEFORE has nodes at 0, 3, 6, 10, 11 and those nodes have height 0, 2, 4, 1, 0. Note that a node
 * does not have an end, it remains in-force until superseded by another node.
 */

use std::fmt::Debug;

use crate::boom::{Boom, BoomCursorMut};

struct WatermarkRequest<'a> {
    iter: BoomCursorMut<'a>,
    start: i64,
    end: i64,
    own_height: f64,
    after_pos: Option<(i64,f64)>,
    max_existing_height: f64,
    pre_start_height: f64,
    final_masked_height: f64
}

impl<'a> WatermarkRequest<'a> {
    fn new(watermark: &'a mut Watermark, start: i64, end: i64, height: f64) -> WatermarkRequest<'a> {
        WatermarkRequest {
            iter: watermark.tree.seek_mut(&start),
            start, end, own_height: height,
            after_pos: None,
            max_existing_height: 0.,
            pre_start_height: 0.,
            final_masked_height: 0.
        }
    }

    /* Investigate the node immediately before our intended start (if any), recording any relevant information
     * for later. Whether such a node is present or not, leave the iterator such that iter.next() will 
     * return the first node at or after out intended start.
     * 
     * We care about the height of this previous node, if any, for three reasons.
     * 
     * 1. Unless an existing node is at our intended start, the previous node will affect the existing maxiumum
     * height in the region, overlapping a new allocation from the left, so we record this in 
     * self.max_existing_heiight. If there does turn out to be a node at exactly at our intended start, we find
     * out about it later and reset self.max_existing_height value to zero before it is futher changed.
     * 
     * 2. At our intended end, unless there is a new start at exactly that position, we need to re-esebalish
     * the last pre-existing height in the region, being the last node under our intended locaiont. If no such
     * node exists, we must reestablish the height before our allocation started, so we record it in 
     * self.final_masked_height, overridden by any node to the right, under our allocation.
     * 
     * 3. If the height of this preceding node matches the ultimately determined height of our new node we 
     * don't need to insert a new entry, we just let the existing node run on. To test for this, 
     * self.pre_start_height is set.
     */
     fn investigate_pre_start(&mut self) {
        if let Some((_,prev_height)) = self.iter.rewind() {
            /* there is a node before start */
            self.iter.next();
            self.max_existing_height = prev_height;
            self.pre_start_height = prev_height;
            self.final_masked_height = prev_height;
        }
    }

    /* Advance the iterator under our intended region, looking for existing nodes to remove, removing them if we should,
     * updating any internal variables as necessary. This method does one step of that process.
     * 
     * Where a node is to be removed (because it sits under our new region):
     * 
     * 1. The removed node ndecssarily has a height and overlaps our target region, so self.max_existing_height
     * needs updating to take that into account, if necessary.
     * 
     * 2. If this is the last start in our intended region, this will be the height to reestablish at the end
     * of our intended region, when we are done. To allow for this, we update self.final_masked_height.
     * 
     * 3. We assumed in investigate_pre_start() that there would be some overlap between that pre-start node (if
     * any) and our intended region. But if there is already a node at exactly our start, coincident with the
     * new node, then the height of the previous region doesn't matter. To effect this, if a node is found at
     * our intended start, self.max_existing_height is reset to zero from the value in investigate_pre_start().
     * 
     * This function runs on to one node at-or-after our new region (but doesn't remove it). When it encounters
     * that node (if any):
     * 
     * 1. If this node is *at* our end rather than *after*, we don't add a new node at our end to establish the
     * pre-existing height.
     * 
     * 2. If, however, the node is *after* our end, we certainly do need to insert an end node. *However* if that
     * existing node is thesame height as our end then that node is deleted. In effect, we "shift" it back to our
     * end position.
     * 
     * We don't do either of these operations, but we record the position and height of this node, where it
     * exists in self.after_end, ready for use. This method doesn't actually do any deleting! It returns the
     * index of the node to be removed (if any). The loop does the deletion.
     */
    fn remove_one_old(&mut self) -> Option<i64> {
        if let Some((next_start,next_height)) = self.iter.next() {
            if next_start < self.end {
                /* node is to be removed */
                self.final_masked_height = next_height;
                if next_start == self.start {
                    /* start coincident with ours so previous node doesn't contribute after all: no overlap */
                    self.max_existing_height = 0.;
                }
                self.max_existing_height = self.max_existing_height.max(next_height);
                return Some(next_start);
            } else {
                /* node which exists but is not to be removed */
                self.after_pos = Some((next_start,next_height));
            }
        }
        None
    }

    /* Loop calling remove_one_old() until it yields no more. See comment on that method for details. */
    fn remove_to_end(&mut self) {
        while let Some(remove) = self.remove_one_old() {
            self.iter.tree().remove(remove);
        }
    }

    /* Insert node at "our" start of calculated final height, unless the pre_start node identified in
     * investigate_pre_start() matches (inwhich case, that range runs on).
     */
    fn update_map_start(&mut self, new_height: f64) {
        if new_height != self.pre_start_height {
            self.iter.tree().insert(self.start,new_height);
        }
    }

    /* Do "the right thing" at the end of our range. The right thing is one of:
     * a. If the next node is directly *at* our end, and *matches* our new height, delete it and we are done.
     * b. Otherwise, if there is a node *at* our end, leave it be.
     * c. If there is no node directly *at* our end, add a node *at* end set to the correct height.
     * d. Following (c), if the subsequent node height (if any) matches the correct height, delete that node.
     */
    fn update_map_end(&mut self, new_height: f64) {
        if let Some((after_start,after_height)) = self.after_pos {
            /* There *is* a node after ours */
            if after_start == self.end {
                /* cases a&b: the node is *at* our end */
                if new_height == after_height {
                    /* case a: it matches the new intended height */
                    self.iter.tree().remove(after_start);
                }
                return;
            }
        }
        /* cases c&d: no node directly *at* our end, add it */
        self.iter.tree().insert(self.end,self.final_masked_height);
        if let Some((after_start,after_height)) = self.after_pos {
            /* There *is* a node after ours */
            if after_height == self.final_masked_height {
                /* case d: node after ours hassame height, delete it. */
                self.iter.tree().remove(after_start);
            }
        }
    }

    /* The only method other than the constructor called externally! */
    fn add(&mut self) -> f64 {
        self.investigate_pre_start();
        self.remove_to_end();
        let new_height = self.max_existing_height + self.own_height;
        self.update_map_start(new_height);
        self.update_map_end(new_height);
        self.max_existing_height
    }
}

pub struct Watermark {
    tree: Boom,
    max_height: f64
}

#[cfg(any(debug_assertions,test))]
impl Debug for Watermark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let all = self.readout_map().iter().map(|(start,height)| format!("({},{})",start,height)).collect::<Vec<_>>();
        write!(f,"[{}]",all.join(", "))
    }
}

impl Watermark {
    pub fn new() -> Watermark {
        Watermark {
            tree: Boom::new(),
            max_height: 0.
        }
    }

    pub fn add(&mut self, start: i64, end: i64, height: f64) -> f64 {
        let mut req = WatermarkRequest::new(self,start,end,height);
        let offset = req.add();
        drop(req);
        self.max_height = self.max_height.max(offset+height);
        offset
    }

    pub fn max_height(&self) -> f64 { self.max_height }

    #[cfg(any(debug_assertions,test))]
    fn readout_map(&self) -> Vec<(i64,f64)> {
        self.tree.all().iter().map(|(k,v)| (*k,*v)).collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod test {
    use super::Watermark;

    #[test]
    fn test_watermark_smoke() {
        let mut watermark = Watermark::new();
        println!("{:?}",watermark);
        assert_eq!(watermark.readout_map(),vec![]);
        assert_eq!(0.,watermark.add(5,12,3.));
        println!("{:?}",watermark);
        assert_eq!(watermark.readout_map(),vec![(5,3.),(12,0.)]);
        assert_eq!(3.,watermark.add(2,6,2.));
        println!("{:?}",watermark);
        assert_eq!(watermark.readout_map(),vec![(2,5.),(6,3.),(12,0.)]);
        assert_eq!(3.,watermark.add(6,8,2.));
        println!("{:?}",watermark);
        assert_eq!(watermark.readout_map(),vec![(2,5.),(8,3.),(12,0.)]);
        assert_eq!(0.,watermark.add(0,1,2.));
        println!("{:?}",watermark);
        assert_eq!(watermark.readout_map(),vec![(0,2.),(1,0.),(2,5.),(8,3.),(12,0.)]);
        assert_eq!(3.,watermark.add(9,14,1.));
        println!("{:?}",watermark);
        assert_eq!(watermark.readout_map(),vec![(0,2.),(1,0.),(2,5.),(8,3.),(9,4.),(14,0.)]);
        assert_eq!(5.,watermark.add(7,13,1.));
        println!("{:?}",watermark);
        assert_eq!(watermark.readout_map(),vec![(0,2.),(1,0.),(2,5.),(7,6.),(13,4.),(14,0.)]);
        assert_eq!(6.,watermark.add(0,9,4.));
        println!("{:?}",watermark);
        assert_eq!(watermark.readout_map(),vec![(0,10.),(9,6.),(13,4.),(14,0.)]);
    }

    fn test_once(inputs: &[(i64,i64,f64,f64)], outputs: &[(i64,f64)]) {
        let mut watermark = Watermark::new();
        for (start,end,height,offset) in inputs {
            assert_eq!(*offset,watermark.add(*start,*end,*height));
        }
        println!("{:?}",watermark);
        assert_eq!(outputs,watermark.readout_map());
    }

    #[test]
    fn test_max_height() {
        /* 1. meh should take into account previous height if overlapping */
        test_once(&[(0,10,1.,0.),( 9,20,1.,1.)],&[(0,1.),(9,2.),(20,0.)]);
        /* 2. meh should not take into account previous height if not overlapping */
        test_once(&[(0,10,1.,0.),(10,20,1.,0.)],&[(0,1.),(20,0.)]);
        /* 3. meh should be maximum; 4. meh should take into account first, middle & last */
        test_once(&[(0,10,1.,0.),(10,20,2.,0.),(20,30,3.,0.),(0,30,1.,3.)],&[(0,4.),(30,0.)]);
        test_once(&[(0,10,1.,0.),(10,20,3.,0.),(20,30,2.,0.),(0,30,1.,3.)],&[(0,4.),(30,0.)]);
        test_once(&[(0,10,3.,0.),(10,20,1.,0.),(20,30,2.,0.),(0,30,1.,3.)],&[(0,4.),(30,0.)]);
    }

    #[test]
    fn test_prestart() {
        /* 1. if insert matches new region, no new entry is added */
        test_once(&[(0,10,1.,0.),(10,20,1.,0.)],&[(0,1.),(20,0.)]);
        /* 2. if insert doesn't match new region, new entry is added */
        test_once(&[(0,10,1.,0.),(10,20,2.,0.)],&[(0,1.),(10,2.),(20,0.)]);
    }

    #[test]
    fn test_final_height() {
        /* 1. Final height should be pre height if no underlying allocations */
        test_once(&[(0,10,1.,0.),(5,8,1.,1.)],&[(0,1.),(5,2.),(8,1.),(10,0.)]);
        /* 2. Final height should last height if there are underlying allocations */
        test_once(&[(0,10,1.,0.),(1,2,2.,1.),(2,12,3.,1.),(5,8,1.,4.)],&[(0,1.),(1,3.),(2,4.),(5,5.),(8,4.),(12,0.)]);
    }

    #[test]
    fn test_no_previous() {
        /* 1. with no previous, final height should be 0 if none underlying */
        test_once(&[(0,10,1.,0.)],&[(0,1.),(10,0.)]);
        /* 2. with no previous, start should be omitted if hegiht zero */
        test_once(&[(0,10,0.,0.)],&[(10,0.)]);
        /* 3. with no previous, max height should be zero if none underlying */
        test_once(&[(0,10,1.,0.)],&[(0,1.),(10,0.)]);
        /* 4. with no previous, max height should be non-zero if some underlying */
        test_once(&[(5,6,1.,0.),(0,10,1.,1.)],&[(0,2.),(10,0.)]);
    }

    #[test]
    fn test_remove_nodes() {
        /* 1. nodes should be removed if lying in our range if multiple */
        test_once(&[(0,1,1.,0.),(2,3,1.,0.),(4,5,1.,0.),(6,7,1.,0.),(8,9,1.,0.),(2,7,1.,1.)],
                 &[(0,1.),(1,0.),(2,2.),(7,0.),(8,1.),(9,0.)]);
        /* 2. nodes should be removed if lying in our range if single */
        test_once(&[(0,1,1.,0.),(2,3,1.,0.),(8,9,1.,0.),(2,7,1.,1.)],
                 &[(0,1.),(1,0.),(2,2.),(7,0.),(8,1.),(9,0.)]);
        /* 3. nothing should go wrong if nothing in range */
        test_once(&[(0,1,1.,0.),(8,9,1.,0.),(2,7,1.,0.)],
                 &[(0,1.),(1,0.),(2,1.),(7,0.),(8,1.),(9,0.)]);
        /* 4. nodes should be removed if lying in our range if none after */
        test_once(&[(0,1,1.,0.),(2,3,1.,0.),(2,7,1.,1.)],
                 &[(0,1.),(1,0.),(2,2.),(7,0.)]);
        /* 5. first node should be removed if necessar7 */
        test_once(&[(0,1,1.,0.),(2,3,1.,0.),(0,7,1.,1.)],
                 &[(0,2.),(7,0.)]);
    }

    #[test]
    fn test_map_end() {
        /* a. If the next value is at end and matches our new height, delete it and we are done. */
        test_once(&[(5,10,2.,0.),(0,5,1.,0.),(0,5,1.,1.)],&[(0,2.),(10,0.)]);
        /* b. Otherwise, if there is such a value, leave it be. */
        test_once(&[(5,10,2.,0.),(0,5,1.,0.)],&[(0,1.),(5,2.),(10,0.)]);
        /* c. If there is no value, establish the resored value at end. */
        test_once(&[(6,10,2.,0.),(0,5,1.,0.)],&[(0,1.),(5,0.),(6,2.),(10,0.)]);
        /* d. If the subsequent value matches the newly-established value, delete that subsequent value. */
        test_once(&[(5,10,1.,0.),(0,5,1.,0.)],&[(0,1.),(10,0.)]);
    }
}
