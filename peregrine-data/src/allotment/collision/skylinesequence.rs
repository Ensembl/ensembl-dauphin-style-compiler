/* We can only store a finite lingth of skyline around theregion currently being displayed. This means we
 * need to be able to delete regions which is difficult to do efficiently within a skyline-like data
 * strucutre. Instead, we segment the skyline into segments (akin to carriages but not necessarily 1-to-1
 * with them). When a segment is added if we are "too long" a segment is removed from the other end. The
 * size of the segment is a paremeter passed into the sequence constructor, scaling with the carriage size
 * and the number of segments to keep is a compile-time constant.
 * 
 * Parts do not necessarily limit themselves to a single segment, which means that items which are present in
 * some segments can potentially extend into segments which are missing. So that such items are created if
 * the segment is created, we also keep track of such items as overhangs. At all times the invariant is that
 * the overhangs list contains Items neither entirely within nor entirely outside the region covered by
 * segments.
 * 
 * We allow attempts to add items multiply. Adding to an existing item divides into multiple cases depending
 * whether either the range is extended or the height extended or both. There are three potential outcomes:
 * a. The sequence is extended internally if necessary, with no external consequences;
 * b. The sequence is extended internally with an increase in total height.
 * c. The sequence cannot be updated.
 */

use std::{collections::{VecDeque, HashMap}, ops::Range, sync::Arc, mem};

use peregrine_toolkit::skyline::Skyline;

use crate::allotment::style::allotmentname::AllotmentName;

const SEGMENT_FLANK : usize = 3; /* Keep no more than 2n+2 segments */

pub enum BumpAddOutcome {
    Unmodified(f64),
    NewHeight(f64,i64),
    Failed
}

struct Item {
    name: AllotmentName,
    interval: Range<i64>,
    height: f64,
    offset: f64
}

impl Item {
    fn overlaps(&self, seq: &SkylineSequence, index: usize) -> bool {
        let index_left = seq.start(index);
        let index_right = seq.start(index+1);
        !(self.interval.start >= index_right || self.interval.end <= index_left)
    }

    fn contained_within(&self, seq: &SkylineSequence, left: usize, right: usize) -> bool {
        let index_left = seq.start(left);
        let index_right = seq.start(right+1);
        self.interval.start >= index_left && self.interval.end <= index_right
    }
}

struct Segment {
    index: usize,
    skyline: Skyline,
    items: HashMap<AllotmentName,Arc<Item>>
}

impl Segment {
    fn new(index: usize) -> Segment {
        Segment {
            index,
            skyline: Skyline::new(),
            items: HashMap::new()
        }
    }
    
    fn add_overhang(&self, item: &Arc<Item>) {
        todo!()
    }

    fn convert_to_overhang(&self, seq: &mut SkylineSequence, left: usize, right: usize) {
        for item in self.items.values() {
            /* overhangs have impact on the current region but not entirely contained in deleted region */
            if item.contained_within(seq,left,right) && !item.contained_within(seq,left,left) {
                seq.overhang.insert(item.name.clone(),item.clone());
            }
        }
    }
}

pub struct SkylineSequence {
    segment_size: i64,
    leftmost_segment: Option<usize>,
    segments: VecDeque<Segment>, // "back" is left; "front" is right
    overhang: HashMap<AllotmentName,Arc<Item>>
}

impl SkylineSequence {
    pub fn new(segment_size: i64) -> SkylineSequence {
        SkylineSequence {
            segment_size,
            leftmost_segment: None,
            segments: VecDeque::new(),
            overhang: HashMap::new()
        }
    }

    fn start(&self, index: usize) -> i64 {
        index as i64*self.segment_size
    }

    fn segment_present(&self, index: usize) -> bool {
        if let Some(leftmost_segment) = self.leftmost_segment {
            index >= leftmost_segment && index < leftmost_segment + self.segments.len()
        } else {
            false
        }
    }

    fn delete_segment(&mut self, segment: Segment) {
        let left = if let Some(left) = self.leftmost_segment { left } else { return; };
        let right = left + self.segments.len();
        segment.convert_to_overhang(self,left,right);
    }

    fn prune_overhangs(&mut self) {
        let leftmost_segment = if let Some(left) = self.leftmost_segment { left } else { return; };
        let mut old_items = mem::replace(&mut self.overhang,HashMap::new());
        for (name,item) in old_items.drain() {
            if !item.contained_within(self,leftmost_segment,leftmost_segment+self.segments.len()-1) {
                self.overhang.insert(name,item);
            }
        }
    }

    fn new_segment(&mut self, index: usize) -> Segment {
        let segment = Segment::new(index);
        /* Apply overhangs to new segment */
        for (_,item) in self.overhang.iter() {
            if item.overlaps(self,index) {
                segment.add_overhang(item);
            }
        }
        segment
    }

    fn remove_unwanted(&mut self, centre: usize) {
        while self.leftmost_segment.map(|left| left < centre - SEGMENT_FLANK).unwrap_or(false) {
            /* remove from left */
            let segment = self.segments.pop_back().unwrap();
            if self.segments.len() == 0 {
                self.leftmost_segment = None;
            } else {
                *self.leftmost_segment.as_mut().unwrap() += 1;            
            }
            self.delete_segment(segment);
        }
        while self.leftmost_segment.map(|left| left + self.segments.len() > centre + SEGMENT_FLANK).unwrap_or(false) {
            /* remove from right */
            let segment = self.segments.pop_front().unwrap();
            if self.segments.len() == 0 { self.leftmost_segment = None; }
            self.delete_segment(segment);
        }
    }

    fn add_segment(&mut self, centre_segment: usize) {
        /* Add to left or right or start over ... */
        let len = self.segments.len();
        if let Some(leftmost_segment) = self.leftmost_segment {
            if centre_segment == leftmost_segment - 1 {
                /* left */
                let segment = self.new_segment(leftmost_segment-1);
                self.segments.push_back(segment);
                *self.leftmost_segment.as_mut().unwrap() -= 1;
                return;
            }
            if centre_segment == leftmost_segment + self.segments.len() {
                /* right */
                let segment = self.new_segment(leftmost_segment+len);
                self.segments.push_front(segment);
                return;
            }
        }
        /* start over */
        while let Some(segment) = self.segments.pop_back() {
            self.delete_segment(segment);
        }
        self.overhang = HashMap::new();
        self.leftmost_segment = Some(centre_segment);
        let segment = self.new_segment(centre_segment);
        self.segments.push_front(segment);
    }

    pub fn set_centre(&mut self, position: i64) {
        let centre_segment = (position / self.segment_size) as usize;
        if self.segment_present(centre_segment) { return; }
        self.add_segment(centre_segment);
        self.prune_overhangs();
    }

    pub fn add(&mut self, name: AllotmentName, interval: Range<i64>, height: f64) -> BumpAddOutcome {
        todo!()
    }
}
