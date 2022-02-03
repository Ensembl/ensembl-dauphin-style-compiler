use std::fmt::Debug;

/* We don't need any of bplustree's fancy locking, but it's themost complete B+ Tree impl for rust. If those locks
 * slow things down too much, we might need to roll our own. The opswe would need would be:
 * goto-first; prev; next; seek; insert; delete.
 */

use bplustree::{iter::RawExclusiveIter, GenericBPlusTree};

const IC : usize = 8;
const LC : usize = 16;

struct WatermarkRequest<'a> {
    iter: RawExclusiveIter<'a,i64,f64,IC,LC>,
    start: i64,
    end: i64,
    own_height: f64,
    after_pos: Option<i64>,
    max_existing_height: f64,
    pre_start_height: f64,
    final_masked_height: f64
}

impl<'a> WatermarkRequest<'a> {
    fn new(watermark: &'a mut Watermark, start: i64, end: i64, height: f64) -> WatermarkRequest<'a> {
        WatermarkRequest {
            iter: watermark.tree.raw_iter_mut(),
            start, end, own_height: height,
            after_pos: None,
            max_existing_height: 0.,
            pre_start_height: 0.,
            final_masked_height: 0.
        }
    }

    /* Investigate the node before the place we will put start (if any), recording any pertainent information for later.
     * Present or not, leave the iterator such that iter.next() will return the first node at or after start.
     * 
     * We care about the height of this pre-node, if any, for three reasons.
     * 
     * 1. Unless there is an node already at start, this will be a contender for the maxiumum height in the region, 
     * overlapping a new allocation from the left, so we record this in self.max_existing_heiight. If there does turn out
     * to be a node at exactly start, we find out about it later and reset its value to zero.
     * 
     * 2. After we have finished, unless there is a new start at our end, we need to re-esebalish the last prevailing
     * height in the region, being the last to start underneath it. If no such region exists, we must reestablish the
     * height before our region started, so wer record it in self.final_masked_height, overridden by any later node.
     * 
     * 3. If the height of this node matches the ultimately determined height of our new region we don't need to insert
     * a new entry, we just let it run on. To test for this, self.pre_start_height is set.
     */
    fn investigate_pre_start(&mut self) {
        self.iter.seek(&self.start);
        if let Some(prev_height) = self.iter.prev().map(|(_,h)| *h) {
            /* there is a node before start */
            self.iter.next();
            self.max_existing_height = prev_height;
            self.pre_start_height = prev_height;
            self.final_masked_height = prev_height;
        }
    }

    /* Advance the iterator looking for nodes to remove and remove them if we should, updating any internal variables
     * as necessary.
     * 
     * In detail, we schedule for removal those nodes which start before our end. In this case we need to update three
     * things:
     * 
     * 1. This node has a height and overlaps our target region, so self.max_existing_height needs updating to take that
     * into account, if necessary.
     * 
     * 2. This could be the last start in our resion and so be the value which we may need to reestablish at our end,
     * when we are done. To allow for this, we update self.final_masked_height.
     * 
     * 3. We assumed at the start that there would be some overlap between the pre-start node (if any) and our region.
     * If the existing allocations actually already had a node starting at start, coincident with the new node, then the
     * previous region doesn't matter. To guard for this, if this node is at the start self.max_existing_height is reset.
     * 
     * If we actually find a node at or after our end, there are two things we need to check about it:
     * 
     * 1. We need to record its position. If it is co-incident with our end, we don't add an end range to the map for
     * our region because a new region starts immediately.
     * 
     * 2. Otherwise, if its height matches the height we wish to reestablish at end, though we do need to add our new
     * end position, we can delete this later node.
     * 
     * To achieve these later, the position and height of this node, whereit exists, are recorded in self.after_end.
     */
    fn remove_one_old(&mut self) -> Option<i64> {
        if let Some((next_start,next_height)) = self.iter.next() {
            if *next_start < self.end {
                /* node is to be removed */
                self.final_masked_height = *next_height;
                if *next_start == self.start {
                    /* start coincident with ours so previous node doesn't contribute after all: no overlap */
                    self.max_existing_height = 0.;
                }
                self.max_existing_height = self.max_existing_height.max(*next_height);
                return Some(*next_start);
            } else {
                /* node which exists but is not to be removed */
                self.after_pos = Some(*next_start);
            }
        }
        None
    }

    /* call remove_one_old until it yields no more.*/
    fn remove_to_end(&mut self) {
        while let Some(remove) = self.remove_one_old() {
            self.iter.remove(&remove);
        }
    }

    /* Now we know the final height, we must fixup the map: here relating to the region start.
     * We insert a start node unless our height matches the height of the previous node.
     */
    fn update_map_start(&mut self, new_height: f64) {
        if new_height != self.pre_start_height {
            self.iter.insert(self.start,new_height);
        }
    }

    /* Unless there is a node at end, we will be adding a new range to reesatblish the old height. Do we need
     * to worry about this equalling the value of the next range after our insert (call it X)? The value we use for our
     * new insert is always that of the final start before our range end. Before we started, this would have been the
     * value immediately before X, so there would have already been a violation of the invariant (should have been
     * merged). So this cannot happen.
     * 
     * The only thing we need worry about is if the next element starts at our end which means we shouldn't add a
     * reestablishing range.
     */
    fn update_map_end(&mut self,) {
        if let Some(after_start) = self.after_pos {
            /* does a range already start at end? */
            if after_start == self.end { return; }
        }
        self.iter.insert(self.end,self.final_masked_height);
    }

    fn add(&mut self) -> f64 {
        self.investigate_pre_start();
        self.remove_to_end();
        let new_height = self.max_existing_height + self.own_height;
        self.update_map_start(new_height);
        self.update_map_end();
        self.max_existing_height
    }
}

struct Watermark {
    tree: GenericBPlusTree<i64,f64,IC,LC>,
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
    fn new() -> Watermark {
        Watermark {
            tree: GenericBPlusTree::new(),
            max_height: 0.
        }
    }

    fn add(&mut self, start: i64, end: i64, height: f64) -> f64 {
        let mut req = WatermarkRequest::new(self,start,end,height);
        let offset = req.add();
        drop(req);
        self.max_height = self.max_height.max(offset+height);
        offset
    }

    fn max_height(&self) -> f64 { self.max_height }

    #[cfg(any(debug_assertions,test))]
    fn readout_map(&self) -> Vec<(i64,f64)> {
        let mut members = vec![];
        let mut iter = self.tree.raw_iter();
        iter.seek_to_first();
        while let Some((start,height)) = iter.next() {
            members.push((*start,*height));
        }
        members
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
        test_once(&[(0,10,1.,0.),(10,20,1.,1.)],&[(0,1.),(20,0.)]);
        /* 2. if insert doesn't match new region, new entry is added */
        test_once(&[(0,10,1.,0.),(10,20,2.,1.)],&[(0,1.),(10,2.),(20,0.)]);
    }

    #[test]
    fn test_final_height() {
        /* 1. Final height should be pre height if no underlying allocations */
        test_once(&[(0,10,1.,0.),(5,8,1.,1.)],&[(0,1.),(5,2.),(8,1.),(10,0.)]);
        /* 2. Final height should last height if there are underlying allocations */
        test_once(&[(0,10,1.,0.),(1,2,2.,1.),(2,12,3.,1.),(5,8,1.,1.)],&[(0,1.),(1,3.),(2,4.),(5,5.),(8,3.),(12,0.)]);
    }

    #[test]
    fn test_no_previous() {
        /* 1. with no previous, final height should be 0 if none underlying */
        test_once(&[(0,10,1.,0.)],&[(0,1.),(10,0.)]);
        /* 2. with no previous, start should be omitted if hegiht zero */
        test_once(&[(0,10,0.,0.)],&[]);
        /* 3. with no previous, max height should be zero if none underlying */
        test_once(&[(0,10,1.,0.)],&[(0,1.),(10,0.)]);
        /* 4. with no previous, max height should be non-zero if some underlying */
        test_once(&[(5,6,1.,0.),(0,10,1.,1.)],&[(0,1.),(5,2.),(6,1.),(10,0.)]);
    }

    #[test]
    fn test_remove_nodes() {
        /* 1. nodes should be removed if lying in our range if multiple */
        test_once(&[(0,1,1.,0.),(2,3,1.,0.),(4,5,1.,0.),(6,7,1.,0.),(8,9,1.,0.),(2,7,1.,1.)],
                 &[(0,1.),(1,0.),(2,2.),(7,0.),(8,1.),(9,0.)]);
        /* 2. nodes should be removed if lying in our range if single */
        test_once(&[(0,1,1.,0.),(2,3,1.,0.),(8,9,1.,0.),(2,7,1.,1.)],
                 &[(0,1.),(1,0.),(2,2.),(7,0.),(8,1.),(9,0.)]);
        /* 3. nothing should go wrong if nothing in gange */
        test_once(&[(0,1,1.,0.),(8,9,1.,0.),(2,7,1.,1.)],
                 &[(0,1.),(1,0.),(2,2.),(7,0.),(8,1.),(9,0.)]);
        /* 4. nodes should be removed if lying in our range if none after */
        test_once(&[(0,1,1.,0.),(2,3,1.,0.),(2,7,1.,1.)],
                 &[(0,1.),(1,0.),(2,2.),(7,0.)]);
        /* 5. first node should be removed if necessar7 */
        test_once(&[(0,1,1.,0.),(2,3,1.,0.),(0,7,1.,1.)],
                 &[(0,2.),(7,0.)]);
    }

    #[test]
    fn test_map_end() {
        /* 1. if there's a node exactly at our end don't add final height */
        test_once(&[(5,10,1.,0.),(0,5,1.,0.)],&[(0,1.),(10,0.)]);
        /* 2. otherwise, if there's a node after, add in final height */
        test_once(&[(5,10,1.,0.),(0,4,1.,0.)],&[(0,1.),(4,0.),(5,1.),(10,0.)]);
        /* 3. if no node after, add in final height */
        test_once(&[(5,10,1.,0.)],&[(0,1.),(10,0.)]);
    }
}
