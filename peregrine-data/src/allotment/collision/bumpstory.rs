/*
 * A BumpRequests contains a list of items from a carriage which need to be bumped. A BumpStory
 * accommodates a lit of contiguous BumpRequestss and, when doing so, bumps them. You can then use
 * the BumpStory to answer the BumpingOffset question.
 * 
 * BumpStory uses SlidingWindow to maintain the list of contiguous BumpRequests and to give the
 * relevant context when adding a new one, to allow it to be bumped. The bumping uses Castles. The Castle
 * is first created by the adjacent BumpRequests (if any) and then passes through the newly added
 * BumpRequests in the relevant direction, adding offsets.
 * 
 * Note that the start and end within a BumpRequest inside any given BumpRequests is guaranteed to
 * contain *all* the region in the carriage for that BumpRequests but not necessarily any more than
 * that. It may contain more than that, up to the entire allocation, but there is no guarantee of that.
 */

use std::{collections::{HashMap}};

use crate::allotment::{style::allotmentname::AllotmentName, collision::castle::Castle};

use super::slidingwindow::{SlidingWindow, SlidingWindowContext};

#[derive(Clone)]
struct BumpRequest {
    start: u64,
    end: u64,
    height: i64
}

impl BumpRequest {
    fn try_add_to_overhang_castle(&self, results: &BumpResults, name: &AllotmentName, castle: &mut Castle, cut_off: u64, hanging_left: bool) {
        if hanging_left && self.start < cut_off {
            let offset = results.items.get(name).cloned().unwrap_or(0);
            castle.allocate_at(self.start,offset,self.height);
        }
        if !hanging_left && self.end >= cut_off {
            let offset = results.items.get(name).cloned().unwrap_or(0);
            castle.allocate_at(self.end,offset,self.height);
        }
    }
}

pub(super) struct BumpRequests {
    bp_per_carriage: u64,
    carriage_index: usize,
    items: HashMap<AllotmentName,BumpRequest>
}

pub(super) struct BumpResults {
    items: HashMap<AllotmentName,i64>
}

impl BumpResults {
    fn get_offset(&self, name: &AllotmentName) -> Option<i64> {
        self.items.get(name).cloned()
    }
}

struct BumpOutcome(BumpRequests,Option<BumpResults>);

impl BumpOutcome {
    fn new(requests: BumpRequests) -> BumpOutcome {
        BumpOutcome(requests,None)
    }

    fn make_castle(&self, hanging_left: bool) -> Castle {
        let delta = if hanging_left { 0 } else { 1 };
        let cut_off = self.0.bp_per_carriage * ((self.0.carriage_index+delta) as u64);
        let mut castle = Castle::new(!hanging_left);
        for (name,item) in self.0.items.iter() {
            item.try_add_to_overhang_castle(&self.1.as_ref().unwrap(),name,&mut castle,cut_off,hanging_left);
        }
        castle
    }

    fn bump(&mut self, companion: Option<&BumpOutcome>, to_left_of_companion: bool) {
        self.1 = Some(self.0.bump(companion,to_left_of_companion));
    }
}

impl BumpRequests {
    fn new(bp_per_carriage: u64, carriage_index: usize) -> BumpRequests {
        BumpRequests {
            bp_per_carriage,
            carriage_index,
            items: HashMap::new()
        }
    }

    pub(super) fn add_item(&mut self, name: &AllotmentName, start: u64, end: u64, height: i64) {
        let item = BumpRequest { start, end, height };
        self.items.insert(name.clone(),item);
    }

    fn bump(&mut self, companion: Option<&BumpOutcome>, to_left_of_companion: bool) -> BumpResults {
        let mut out = BumpResults { items: HashMap::new() };
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
        for (name,item) in order.iter_mut() {
            castle.advance_to(if go_left { item.end } else { item.start });
            let offset = castle.allocate(if go_left { item.start } else { item.end }, item.height);
            out.items.insert(name.clone(),offset);
        }
        out
    }
}

const STORE_LENGTH : usize = 11;

pub(crate) struct BumpStory {
    sliding: SlidingWindow<'static,BumpOutcome>,
    bp_per_carriage: u64,
}

impl BumpStory {
    pub(crate) fn new(bp_per_carriage: u64) -> BumpStory {
        BumpStory {
            bp_per_carriage,
            sliding: SlidingWindow::new(
                STORE_LENGTH,
                |store: &BumpOutcome| store.0.carriage_index,
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

    pub(super) fn make_carriage_bumps(&self, index: usize) -> BumpRequests {
        BumpRequests::new(self.bp_per_carriage,index)
    }

    pub(super) fn add(&mut self, store: BumpRequests) -> bool {
        self.sliding.set_lock(store.carriage_index,true);
        self.sliding.add(BumpOutcome::new(store))
    }

    pub(super) fn unlock(&mut self, carriage_index: usize) {
        self.sliding.set_lock(carriage_index,false);
    }

    pub(super) fn get_results(&self, carriage_index: usize) -> Option<&BumpResults> {
        self.sliding.get(carriage_index).and_then(|x| x.1.as_ref())
    }
}
