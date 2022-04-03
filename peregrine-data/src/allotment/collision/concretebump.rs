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
    finite: HashMap<AllotmentName,ConcreteRequest>,
    infinite: Vec<(AllotmentName,i64)>,
    max_infinite: i64
}

impl ConcreteRequests {
    fn new(bp_per_carriage: u64, carriage_index: usize) -> ConcreteRequests {
        ConcreteRequests {
            bp_per_carriage,
            carriage_index,
            finite: HashMap::new(),
            infinite: vec![],
            max_infinite: 0
        }
    }

    pub(super) fn add_finite(&mut self, name: &AllotmentName, start: u64, end: u64, height: i64) {
        let item = ConcreteRequest { start, end, height };
        self.finite.insert(name.clone(),item);
    }

    pub(super) fn add_infinite(&mut self, name: &AllotmentName, height: i64) {
        self.infinite.push((name.clone(),self.max_infinite));
        self.max_infinite += height;
    }

    fn bump(&mut self, companion: Option<&ConcreteOutcome>, to_left_of_companion: bool, max_infinite: i64) -> ConcreteResults {
        let mut items = HashMap::new();
        /* Get castle due to overhang or make our own */
        let (mut castle,go_left) = if let Some(companion) = companion {
            (companion.make_castle(to_left_of_companion),to_left_of_companion)
        } else {
            (Castle::new(true),false)
        };
        /* In what order do we need to bump? */
        let mut order = self.finite.iter_mut().collect::<Vec<_>>();
        order.sort_by_cached_key(|(_,item)| {
            if go_left { item.end } else { item.start }
        });
        /* Add our items and set offset */
        for (name,item) in order.iter_mut() {
            castle.advance_to(if go_left { item.end } else { item.start });
            let offset = castle.allocate(if go_left { item.start } else { item.end }, item.height);
            items.insert(name.clone(),max_infinite+offset);
        }
        /* Add infinite items */
        for (name,offset) in &self.infinite {
            items.insert(name.clone(),*offset);
        }
        ConcreteResults { items, maximum: castle.maximum() }
    }
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

struct ConcreteOutcome {
    requests:ConcreteRequests,
    max_infinite: i64,
    results: Option<ConcreteResults>
}

impl ConcreteOutcome {
    fn new(requests: ConcreteRequests, max_infinite: i64) -> ConcreteOutcome {
        ConcreteOutcome { requests, max_infinite, results: None }
    }

    fn make_castle(&self, hanging_left: bool) -> Castle {
        let delta = if hanging_left { 0 } else { 1 };
        let cut_off = self.requests.bp_per_carriage * ((self.requests.carriage_index+delta) as u64);
        let mut castle = Castle::new(!hanging_left);
        for (name,item) in self.requests.finite.iter() {
            item.try_add_to_overhang_castle(&self.results.as_ref().unwrap(),name,&mut castle,cut_off,hanging_left);
        }
        castle
    }

    fn bump(&mut self, companion: Option<&ConcreteOutcome>, to_left_of_companion: bool) -> i64 {
        let result = self.requests.bump(companion,to_left_of_companion,self.max_infinite);
        let maximum = result.maximum + self.max_infinite;
        self.results = Some(result);
        maximum
    }
}

const STORE_LENGTH : usize = 11;

pub(crate) struct ConcreteBump {
    sliding: SlidingWindow<'static,ConcreteOutcome,i64>,
    bp_per_carriage: u64,
    max_infinite: Option<i64>,
    maximum: i64
}

impl ConcreteBump {
    pub(crate) fn new(bp_per_carriage: u64) -> ConcreteBump {
        ConcreteBump {
            bp_per_carriage,
            maximum: 0,
            max_infinite: None,
            sliding: SlidingWindow::new(
                STORE_LENGTH,
                |store: &ConcreteOutcome| store.requests.carriage_index,
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

    pub(super) fn new_requests(&self, index: usize) -> ConcreteRequests {
        ConcreteRequests::new(self.bp_per_carriage,index)
    }

    pub(super) fn add(&mut self, store: ConcreteRequests) -> bool {
        /* check infinite size is compatible */
        if let Some(infinite) = self.max_infinite {
            if infinite < store.max_infinite { return false; }
        } else {
            self.max_infinite = Some(store.max_infinite);
        }
        /* check if we already have it */
        if self.sliding.get(store.carriage_index).is_some() {
            self.sliding.set_lock(store.carriage_index,true);
            return true
        }
        /* add */
        let carriage_index = store.carriage_index;
        if let Some(maximum) = self.sliding.add(ConcreteOutcome::new(store,self.max_infinite.unwrap())) {
            self.maximum = self.maximum.max(maximum);
            self.sliding.set_lock(carriage_index,true);
            true
        } else {
            false
        }
    }

    pub(super) fn try_lock(&mut self, carriage_index: usize) -> bool {
        if self.sliding.get(carriage_index).is_some() {
            self.sliding.set_lock(carriage_index,true);
            true
        } else {
            false
        }
    }

    pub(super) fn release(&mut self, carriage_index: usize) {
        self.sliding.set_lock(carriage_index,false);
    }

    pub(super) fn get_results(&self, carriage_index: usize) -> Option<&ConcreteResults> {
        self.sliding.get(carriage_index).and_then(|x| x.results.as_ref())
    }
}
