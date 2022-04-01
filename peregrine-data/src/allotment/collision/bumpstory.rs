/*
 * A CarriageBumpItemStore contains a list of items from a carriage which need to be bumped. A BumpStory
 * accommodates a lit of contiguous CarriageBumpItemStores and, when doing so, bumps them. You can then use
 * the BumpStory to answer the BumpingOffset question.
 * 
 * BumpStory uses SlidingWindow to maintain the list of contiguous CarriageBumpItemStores and to give the
 * relevant context when adding a new one, to allow it to be bumped. The bumping uses Castles. The Castle
 * is first created by the adjacent CarriageBumpItemStore (if any) and then passes through the newly added
 * CarriageBumpItemStore in the relevant direction, adding offsets.
 * 
 * Note that the start and end within a BumpItem inside any given CarriageBumpItemStore is guaranteed to
 * contain *all* the region in the carriage for that CarriageBumpItemStore but not necessarily any more than
 * that. It may contain more than that, up to the entire allocation, but there is no guarantee of that.
 */

use std::{collections::{HashMap}};

use crate::allotment::{style::allotmentname::AllotmentName, collision::castle::Castle};

use super::slidingwindow::{SlidingWindow, SlidingWindowContext};

#[derive(Clone)]
struct BumpItem {
    start: i64,
    end: i64,
    offset: i64,
    height: i64
}

impl BumpItem {
    fn try_add_to_overhang_castle(&self, castle: &mut Castle, cut_off: i64, hanging_left: bool) {
        if hanging_left && self.start < cut_off {
            castle.allocate_at(self.start,self.offset,self.height);
        }
        if !hanging_left && self.end >= cut_off {
            castle.allocate_at(self.end,self.offset,self.height);
        }
    }
}

pub(super) struct CarriageBumpItemStore {
    bp_per_carriage: i64,
    carriage_index: usize,
    items: HashMap<AllotmentName,BumpItem>
}

impl CarriageBumpItemStore {
    fn new(bp_per_carriage: i64, carriage_index: usize) -> CarriageBumpItemStore {
        CarriageBumpItemStore {
            bp_per_carriage,
            carriage_index,
            items: HashMap::new()
        }
    }

    pub(super) fn add_item(&mut self, name: &AllotmentName, start: i64, end: i64, height: i64) {
        let item = BumpItem { start, end, offset: 0 , height };
        self.items.insert(name.clone(),item);
    }

    fn make_castle(&self, hanging_left: bool) -> Castle {
        let delta = if hanging_left { 0 } else { 1 };
        let cut_off = self.bp_per_carriage * ((self.carriage_index+delta) as i64);
        let mut castle = Castle::new(!hanging_left);
        for item in self.items.values() {
            item.try_add_to_overhang_castle(&mut castle,cut_off,hanging_left);
        }
        castle
    }

    fn bump(&mut self, companion: Option<&CarriageBumpItemStore>, to_left_of_companion: bool) {
        /* Get castle due to overhang or make our own */
        let (mut castle,go_left) = if let Some(companion) = companion {
            (companion.make_castle(to_left_of_companion),to_left_of_companion)
        } else {
            (Castle::new(true),false)
        };
        /* In what order do we need to bump? */
        let mut order = self.items.iter_mut().collect::<Vec<_>>();
        order.sort_by_cached_key(|(_,item)| {
            if go_left { item.end } else { item.start }
        });
        /* Add our items and set offset */
        for (_,item) in order.iter_mut() {
            item.offset = castle.allocate(if go_left { item.start } else { item.end }, item.height);
        }
    }

    fn get_offset(&self, name: &AllotmentName) -> Option<i64> {
        self.items.get(name).map(|item| item.offset)
    }
}

const STORE_LENGTH : usize = 11;

pub(super) struct BumpStory {
    sliding: SlidingWindow<'static,CarriageBumpItemStore>,
    bp_per_carriage: i64,
}

impl BumpStory {
    pub(super) fn new(bp_per_carriage: i64) -> BumpStory {
        BumpStory {
            bp_per_carriage,
            sliding: SlidingWindow::new(
                STORE_LENGTH,
                |store: &CarriageBumpItemStore| store.carriage_index,
                |ctx| {
                    match ctx {
                        SlidingWindowContext::Fresh(store) => store.bump(None,false),
                        SlidingWindowContext::Left(store,left) => store.bump(Some(left),true),
                        SlidingWindowContext::Right(right,store) => store.bump(Some(right),false)
                    } 
                },
                |store| {}
            ),
        }
    }

    pub(super) fn make_store(&self, index: usize) -> CarriageBumpItemStore {
        CarriageBumpItemStore::new(self.bp_per_carriage,index)
    }

    pub(super) fn add(&mut self, store: CarriageBumpItemStore) ->bool {
        self.sliding.set_lock(store.carriage_index,true);
        self.sliding.add(store)
    }

    pub(super) fn unlock(&mut self, carriage_index: usize) {
        self.sliding.set_lock(carriage_index,false);
    }

    pub(super) fn get_offset(&self, carriage_index: usize, name: &AllotmentName) -> Option<i64> {
        self.sliding.get(carriage_index).and_then(|store| store.get_offset(name))
    }
}
