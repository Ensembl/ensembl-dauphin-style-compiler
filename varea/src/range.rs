use super::axis::{ VareaIndex, VareaIndexItem, VareaIndexRemover };
use super::walkers::{ NeverVareaWalker, OrVareaSearch };
use super::core::{ VareaId, VareaWalker, VareaSearch };
use std::any::Any;
use std::cell::RefCell;
use std::cmp;
use std::collections::{ BTreeMap };
use std::rc::Rc;

const RTREE_MERGE : u32 = 4;

#[derive(Clone,Debug)]
pub struct RTreeRange(u64,u64);

impl RTreeRange {
    pub fn new(min: u64, max: u64) -> RTreeRange {
        RTreeRange(min,max)
    }
}

impl VareaIndexItem for RTreeRange {
    fn factory_id(&self) -> &str { "range" }
    fn make_index(&self) -> Box<dyn VareaIndex> {
        Box::new(RTree::new(RTREE_MERGE))
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}

struct RTreeNode {
    members: Rc<RefCell<BTreeMap<VareaId,RTreeRange>>>
}

impl RTreeNode {
    fn new() -> RTreeNode {
        RTreeNode {
            members: Rc::new(RefCell::new(BTreeMap::new()))
        }
    }

    fn add(&mut self, r: RTreeRange, value: VareaId) -> RTreeIndexRemover {
        self.members.borrow_mut().insert(value,r);
        RTreeIndexRemover(Some((self.members.clone(),value)))
    }

    fn overlap(&self, r: &RTreeRange) -> VareaSearch {
        Box::new(RTreeNodeVareaWalker(self.members.clone(),r.clone()))
    }
}

pub struct RTreeIndexRemover(Option<(Rc<RefCell<BTreeMap<VareaId,RTreeRange>>>,VareaId)>);

impl VareaIndexRemover for RTreeIndexRemover {
    fn remove(&mut self) {
        if let Some((tree,id)) = &self.0 {
            tree.borrow_mut().remove(&id);
        }
    } 
}

#[cfg_attr(test, derive(Debug))]
pub struct RTreeNodeVareaWalker(Rc<RefCell<BTreeMap<VareaId,RTreeRange>>>,RTreeRange);

impl VareaWalker for RTreeNodeVareaWalker {
    fn next_from(&self, start: VareaId) -> Option<VareaId> {
        self.0.borrow().range(start..).filter(|(_,r)| {
            !(r.1 <= (self.1).0 || r.0 >= (self.1).1)
        }).map(|(k,_)| k).next().cloned()
    }
}

struct RTreeLevel {
    size: u64,
    members: BTreeMap<u64,RTreeNode>
}

impl RTreeLevel {
    fn new(scale: u32) -> RTreeLevel {
        RTreeLevel {
            size: 1<<scale,
            members: BTreeMap::new()
        }
    }

    fn add(&mut self, r: &RTreeRange, value: VareaId) -> RTreeIndexRemover {
        self.members.entry(r.0/self.size).or_insert_with(|| RTreeNode::new()).add(r.to_owned(),value)
    }

    fn overlap(&self, r: &RTreeRange) -> VareaSearch {
        let start = r.0/self.size;
        let end = cmp::max((cmp::max(r.1,1)-1)/self.size+1,start+1);
        if r.1 == 0 { return OrVareaSearch::new(vec![]); }
        OrVareaSearch::new(self.members.range(start..end).map(|(_,v)| v.overlap(r)).collect())
    }
}

// TODO no prior-construction on lookup()

pub struct RTree {
    merge: u32,
    levels: Vec<RTreeLevel>
}

impl RTree {
    pub fn new(merge: u32) -> RTree {
        RTree {
            merge,
            levels: vec![]
        }
    }

    fn correct_level(&self, a: u64, b: u64) -> u32 {
        let mut scale = (b-a).next_power_of_two().trailing_zeros();
        scale = scale / self.merge;
        let chosen_size = 1 << (scale * self.merge);
        if a / chosen_size != (b-1) / chosen_size {
            scale += 1;
        }
        scale
    }

    fn overlap(&self, r: &RTreeRange) -> VareaSearch {
        OrVareaSearch::new(self.levels.iter().map(|x| x.overlap(r)).collect())
    }
}

impl VareaIndex for RTree {
    fn add(&mut self, id: &VareaId, item: Box<dyn VareaIndexItem>) -> Box<dyn VareaIndexRemover> {
        if let Ok(r) = item.into_any().downcast::<RTreeRange>() {
            let slot = self.correct_level(r.0,r.1);
            while slot >= self.levels.len() as u32 {
                self.levels.push(RTreeLevel::new(self.levels.len() as u32 + self.merge));
            }
            Box::new(self.levels[slot as usize].add(&r,*id))
        } else {
            Box::new(RTreeIndexRemover(None))
        }
    }

    fn lookup(&self, area: &Box<dyn VareaIndexItem>) -> VareaSearch {
        if let Some(r) = area.as_any().downcast_ref::<RTreeRange>() {
            if r.0 == r.1 {
                Box::new(NeverVareaWalker())
            } else {
                self.overlap(r)
            }
        } else {
            Box::new(NeverVareaWalker())   
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all(vs: &VareaSearch) -> Vec<VareaId> {
        let mut out = vec![];
        let mut start = 0;
        while let Some(value) = vs.next_from(start) {
            out.push(value);
            start = value+1;
        }
        out
    }

    #[test]
    fn correct_level_test() {
        let rtree = RTree::new(1);
        assert_eq!(4,rtree.correct_level(1,9));
        assert_eq!(2,rtree.correct_level(8,11));
        assert_eq!(0,rtree.correct_level(0,1));
        let rtree2 = RTree::new(2);
        assert_eq!(2,rtree2.correct_level(1,9));
        assert_eq!(1,rtree2.correct_level(8,11));
        assert_eq!(0,rtree2.correct_level(0,1));
    }

    #[test]
    fn node() {
        let mut node = RTreeNode::new();
        node.add(RTreeRange::new(2,10),2);
        node.add(RTreeRange::new(3,12),3);
        node.add(RTreeRange::new(7,25),4);
        let r : VareaSearch = node.overlap(&RTreeRange::new(8,9));
        assert_eq!(vec![2,3,4],all(&r));
        let r : VareaSearch = node.overlap(&RTreeRange::new(2,3));
        assert_eq!(vec![2],all(&r));
        let r : VareaSearch = node.overlap(&RTreeRange::new(3,6));
        assert_eq!(vec![2,3],all(&r));
        let r : VareaSearch = node.overlap(&RTreeRange::new(7,10));
        assert_eq!(vec![2,3,4],all(&r));
        let r : VareaSearch = node.overlap(&RTreeRange::new(11,20));
        assert_eq!(vec![3,4],all(&r));
        let r : VareaSearch = node.overlap(&RTreeRange::new(25,40));
        assert_eq!(Vec::<VareaId>::new(),all(&r));
    }

    #[test]
    fn level() {
        let mut level = RTreeLevel::new(1);
        level.add(&RTreeRange::new(0,2),0);
        level.add(&RTreeRange::new(2,3),1);
        let mut rm = level.add(&RTreeRange::new(3,4),2);
        level.add(&RTreeRange::new(4,6),3);
        level.add(&RTreeRange::new(6,7),4);
        let r : VareaSearch = level.overlap(&RTreeRange::new(0,7));
        assert_eq!(vec![0,1,2,3,4],all(&r));
        let r : VareaSearch = level.overlap(&RTreeRange::new(2,7));
        assert_eq!(vec![1,2,3,4],all(&r));
        let r : VareaSearch = level.overlap(&RTreeRange::new(0,0));
        assert_eq!(Vec::<VareaId>::new(),all(&r));
        let r : VareaSearch = level.overlap(&RTreeRange::new(0,1));
        assert_eq!(vec![0],all(&r));
        let r : VareaSearch = level.overlap(&RTreeRange::new(0,2));
        assert_eq!(vec![0],all(&r));
        let r : VareaSearch = level.overlap(&RTreeRange::new(1,1));
        assert_eq!(vec![0],all(&r));
        let r : VareaSearch = level.overlap(&RTreeRange::new(1,3));
        assert_eq!(vec![0,1],all(&r));
        let r : VareaSearch = level.overlap(&RTreeRange::new(3,3));
        assert_eq!(Vec::<VareaId>::new(),all(&r));
        let r : VareaSearch = level.overlap(&RTreeRange::new(3,4));
        assert_eq!(vec![2],all(&r));
        let r : VareaSearch = level.overlap(&RTreeRange::new(7,8));
        assert_eq!(Vec::<VareaId>::new(),all(&r));
        let r : VareaSearch = level.overlap(&RTreeRange::new(2,3));
        assert_eq!(vec![1],all(&r));
        let r : VareaSearch = level.overlap(&RTreeRange::new(2,4));
        assert_eq!(vec![1,2],all(&r));
        let r : VareaSearch = level.overlap(&RTreeRange::new(3,6));
        assert_eq!(vec![2,3],all(&r));
        let r : VareaSearch = level.overlap(&RTreeRange::new(2,7));
        assert_eq!(vec![1,2,3,4],all(&r));
        rm.remove();
        let r : VareaSearch = level.overlap(&RTreeRange::new(2,7));
        assert_eq!(vec![1,3,4],all(&r));
    }

    fn search(from: u64, to: u64) -> Box<dyn VareaIndexItem> {
        Box::new(RTreeRange::new(from,to))
    }

    #[test]
    fn rtree() {
        let mut tree = RTree::new(1);
        let ranges = vec![(0,2),(2,3),(3,6),(4,6),(4,6),(5,7),(4,8)];
        for (i,(a,b)) in ranges.iter().enumerate() {
            tree.add(&i,Box::new(RTreeRange::new(*a,*b)));
        }
        for start in 0..9 {
            for end in start..10 {
                let mut out = vec![];
                for (i,(a,b)) in ranges.iter().enumerate() {
                    if !(*b <= start || *a >= end  || start==end) {
                        out.push(i);
                    }
                }
                let r : VareaSearch = tree.lookup(&search(start,end));
                assert_eq!(out,all(&r));
        
            }
        }
    }

    #[test]
    fn semi_open() {
        let mut tree = RTree::new(1);
        tree.add(&0,Box::new(RTreeRange::new(2,6)));
        assert_eq!(0,all(&tree.lookup(&search(0,2))).len());
        assert_eq!(0,all(&tree.lookup(&search(1,2))).len());
        assert_eq!(0,all(&tree.lookup(&search(2,2))).len());
        assert_eq!(1,all(&tree.lookup(&search(0,3))).len());
        assert_eq!(1,all(&tree.lookup(&search(1,3))).len());
        assert_eq!(1,all(&tree.lookup(&search(2,3))).len());
        assert_eq!(0,all(&tree.lookup(&search(6,7))).len());
        assert_eq!(0,all(&tree.lookup(&search(6,8))).len());
        assert_eq!(0,all(&tree.lookup(&search(6,9))).len());
        assert_eq!(1,all(&tree.lookup(&search(5,7))).len());
        assert_eq!(1,all(&tree.lookup(&search(5,8))).len());
        assert_eq!(1,all(&tree.lookup(&search(5,9))).len());
    }
}
