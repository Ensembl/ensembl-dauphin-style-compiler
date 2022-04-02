/*
 * A ConcreteRequests contains a list of items from a carriage which need to be bumped. A ConcreteBump
 * accommodates a lit of contiguous ConcreteRequestss and, when doing so, bumps them. You can then use
 * the ConcreteBump to answer the BumpingOffset question.
 * 
 * ConcreteBump uses SlidingWindow to maintain the list of contiguous ConcreteRequests and to give the
 * relevant context when adding a new one, to allow it to be bumped. The bumping uses Castles. The Castle
 * is first created by the adjacent ConcreteRequests (if any) and then passes through the newly added
 * ConcreteRequests in the relevant direction, adding offsets.
 * 
 * Note that the start and end within a ConcreteRequest inside any given ConcreteRequests is guaranteed to
 * contain *all* the region in the carriage for that ConcreteRequests but not necessarily any more than
 * that. It may contain more than that, up to the entire allocation, but there is no guarantee of that.
 */

use std::{collections::{HashMap}};

use crate::allotment::{style::allotmentname::AllotmentName, collision::castle::Castle};

use super::slidingwindow::{SlidingWindow, SlidingWindowContext};

#[derive(Clone)]
struct ConcreteRequest {
    start: u64,
    end: u64,
    height: i64
}

impl ConcreteRequest {
    fn try_add_to_overhang_castle(&self, results: &ConcreteResults, name: &AllotmentName, castle: &mut Castle, cut_off: u64, hanging_left: bool) {
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

pub(super) struct ConcreteRequests {
    bp_per_carriage: u64,
    carriage_index: usize,
    items: HashMap<AllotmentName,ConcreteRequest>
}

pub(super) struct ConcreteResults {
    items: HashMap<AllotmentName,i64>,
    maximum: i64
}

impl ConcreteResults {
    fn get_offset(&self, name: &AllotmentName) -> Option<i64> {
        self.items.get(name).cloned()
    }
}

impl ConcreteRequests {
    fn new(bp_per_carriage: u64, carriage_index: usize) -> ConcreteRequests {
        ConcreteRequests {
            bp_per_carriage,
            carriage_index,
            items: HashMap::new()
        }
    }

    pub(super) fn add_item(&mut self, name: &AllotmentName, start: u64, end: u64, height: i64) {
        let item = ConcreteRequest { start, end, height };
        self.items.insert(name.clone(),item);
    }

    fn bump(&mut self, companion: Option<&ConcreteOutcome>, to_left_of_companion: bool) -> ConcreteResults {
        let mut items = HashMap::new();
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
            items.insert(name.clone(),offset);
        }
        ConcreteResults { items, maximum: castle.maximum() }
    }
}

struct ConcreteOutcome(ConcreteRequests,Option<ConcreteResults>);

impl ConcreteOutcome {
    fn new(requests: ConcreteRequests) -> ConcreteOutcome {
        ConcreteOutcome(requests,None)
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

    fn bump(&mut self, companion: Option<&ConcreteOutcome>, to_left_of_companion: bool) -> i64 {
        let result = self.0.bump(companion,to_left_of_companion);
        let maximum = result.maximum;
        self.1 = Some(result);
        maximum
    }
}

const STORE_LENGTH : usize = 11;

pub(crate) struct ConcreteBump {
    sliding: SlidingWindow<'static,ConcreteOutcome,i64>,
    bp_per_carriage: u64,
    maximum: i64
}

impl ConcreteBump {
    pub(crate) fn new(bp_per_carriage: u64) -> ConcreteBump {
        ConcreteBump {
            bp_per_carriage,
            maximum: 0,
            sliding: SlidingWindow::new(
                STORE_LENGTH,
                |store: &ConcreteOutcome| store.0.carriage_index,
                |ctx| {
                    match ctx {
                        SlidingWindowContext::Fresh(outcome) =>
                            outcome.bump(None,false),
                        SlidingWindowContext::Left(outcome,left) =>
                            outcome.bump(Some(left),true),
                        SlidingWindowContext::Right(right,outcome) =>
                            outcome.bump(Some(right),false)
                    } 
                },
                |store| {}
            ),
        }
    }

    pub(super) fn make_carriage_bumps(&self, index: usize) -> ConcreteRequests {
        ConcreteRequests::new(self.bp_per_carriage,index)
    }

    pub(super) fn add(&mut self, store: ConcreteRequests) -> bool {
        self.sliding.set_lock(store.carriage_index,true);
        if let Some(maximum) = self.sliding.add(ConcreteOutcome::new(store)) {
            self.maximum = self.maximum.max(maximum);
            true
        } else {
            false
        }
    }

    pub(super) fn unlock(&mut self, carriage_index: usize) {
        self.sliding.set_lock(carriage_index,false);
    }

    pub(super) fn get_results(&self, carriage_index: usize) -> Option<&ConcreteResults> {
        self.sliding.get(carriage_index).and_then(|x| x.1.as_ref())
    }
}
