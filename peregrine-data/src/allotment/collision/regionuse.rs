/* INTRODUCTION 
 *
 * The BumpStory is a best-effots algorithm to incrementally accommodate bumped carriages into a consistent
 * block of shapes without "tears". No one carriage necessarily knows the extent of a bumped leaf and they
 * can, in theory, extend arbitrarily both horizontally and vertically beyond the bounds of each carriage.
 * 
 * Sometimes it is not possible to accommodate a new carriage and the merge will fail. At this point the
 * bumpung algorithm must create a new BumpStory with the old items packed differently, but htis should be a
 * rare occurrance. In particular, it will only happen if carriages extend the height of a bumped box.
 *
 * 
 * Every item to be added is represented by one or more uUmpItem structs. A BumpItem is local to the carriage
 * being added. It includes the offset and height of the allocation, along with a range. A range in a BumpItem
 * is guaranteed to be all of, or a subset of the entire range occupied by the leaf, but is guranteed to
 * contain all of the range within the carriagefor that BumpItem. Other BumpItems may wel exist *for the same
 * leaf* in different carriages, possibly containing different ranges.
 * 
 * For exmaple, say a leaf X extends from carriages 7-12. The BumpItem for X in carriage 8 might include an
 * extent from somewhere in carriage 7 to somewhere in carriage 10; its BumpItem in carriage 9 might include
 * only its extent in carriage 9, its BumpItem in carriage 11 its entire extent, etc.
 * 
 * BumpItems are stored in CarriageBumpItemData structs. A number of such structs are stored in the BumpStory.
 * Any already allocated BumpItems must be placed within the Castle of a new carriage before bumping can begin
 * to avoid tearing.
 */

use std::collections::{HashMap};

use crate::allotment::style::allotmentname::AllotmentName;

use super::slidingwindow::SlidingWindow;

struct BumpItem {
    start: i64,
    end: i64,
    offset: i64,
    height: i64
}

pub struct CarriageBumpItemStore {
    carriage_index: usize,
    items: HashMap<AllotmentName,BumpItem>
}

impl CarriageBumpItemStore {
    pub fn new(carriage_index: usize) -> CarriageBumpItemStore {
        CarriageBumpItemStore {
            carriage_index,
            items: HashMap::new()
        }
    }

    pub fn add_item(&mut self, name: &AllotmentName, start: i64, end: i64, offset: i64) {
        let item = BumpItem { start, end, offset, height: 0 };
        self.items.insert(name.clone(),item);
    }
}

const STORE_LENGTH : usize = 5;

pub(super) struct BumpStory {
    sliding: SlidingWindow<'static,CarriageBumpItemStore>,
    bp_per_carriage: i64,
}

impl BumpStory {
    pub fn new(bp_per_carriage: i64) -> BumpStory {
        BumpStory {
            bp_per_carriage,
            sliding: SlidingWindow::new(
                STORE_LENGTH,
                |store: &CarriageBumpItemStore| store.carriage_index,
                |store| {},
                |store| {}
            ),
        }
    }

    pub fn add(&mut self, store: CarriageBumpItemStore) -> bool {
        self.sliding.add(store)
    }
}
