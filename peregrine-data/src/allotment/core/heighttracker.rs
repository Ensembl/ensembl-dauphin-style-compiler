use std::{collections::{HashMap, hash_map::{DefaultHasher}}, sync::{Arc, Mutex}, fmt, hash::{Hash, Hasher}};

use peregrine_toolkit::{error, puzzle::{DelayedCommuteBuilder, UnknownSetter, StaticValue, StaticAnswer, short_unknown_promise_clonable, Answer, cache_constant, commute, short_memoized, commute_arc, short_memoized_arc, cache_constant_arc, constant, short_unknown_function, short_unknown_function_promise}, lock};

use crate::allotment::{style::allotmentname::AllotmentName};

struct HeightTrackerEntry {
    extra_setter: UnknownSetter<'static,f64>,
    extra: StaticValue<f64>,
    used: DelayedCommuteBuilder<'static,f64>
}

impl HeightTrackerEntry {
    fn new(independent_answer: &mut StaticAnswer) -> HeightTrackerEntry {
        let (mut extra_setter,extra) = short_unknown_promise_clonable();
        let used = DelayedCommuteBuilder::new(|a,b| f64::max(*a,*b));
        extra_setter.set(independent_answer,0.);
        HeightTrackerEntry {
            extra_setter, extra, used
        }
    }

    fn add(&mut self, height: &StaticValue<f64>) {
        self.used.add(height);
    }

    fn build(&mut self) {
        self.used.build(0.);
    }

    fn get_used(&self, answer_index: &mut StaticAnswer) -> f64 {
        *self.used.solver().call(answer_index)
    }

    fn set_full_height(&mut self, answer_index: &mut StaticAnswer, full_height: f64) {
        self.extra_setter.set(answer_index,full_height);
    }

    fn get_piece(&self) -> &StaticValue<f64> {
        &self.extra
    }
}

pub struct HeightTrackerPieces {
    heights: Mutex<HashMap<AllotmentName,HeightTrackerEntry>>,

    #[cfg(debug_assertions)]
    built: bool,
}

impl HeightTrackerPieces {
    pub(crate) fn new() -> HeightTrackerPieces {
        HeightTrackerPieces {
            heights: Mutex::new(HashMap::new()),

            #[cfg(debug_assertions)]
            built: false
        }
    }

    fn ensure_entry(&self, name: &AllotmentName, independent_answer: &mut StaticAnswer) {
        let mut heights = lock!(self.heights);
        if !heights.contains_key(name) {
            heights.insert(name.clone(),HeightTrackerEntry::new(independent_answer));
        }
    }

    pub(crate) fn add(&mut self, name: &AllotmentName, height: &StaticValue<f64>, independent_answer: &mut StaticAnswer) {
        self.ensure_entry(name,independent_answer);
        lock!(self.heights).get_mut(name).unwrap().add(height);
    }

    pub(crate) fn build(&mut self) {
        #[cfg(debug_assertions)]
        { self.built = true; }
        for values in lock!(self.heights).values_mut() {
            values.build();
        }
    }

    pub(crate) fn get_piece(&mut self, name: &AllotmentName, independent_answer: &mut StaticAnswer) -> StaticValue<f64> {
        self.ensure_entry(name,independent_answer);
        lock!(self.heights).get(name).unwrap().get_piece().clone()
    }

    /* must be called pre-solve */
    pub(crate) fn set_extra_height(&self, answer_index: &mut StaticAnswer, heights: &HeightTracker) {
        for (name,entry) in lock!(self.heights).iter_mut() {
            entry.set_full_height(answer_index,heights.get(name));
        }
    }
}

pub struct HeightTrackerMerger {
    heights: HashMap<AllotmentName,i64>
}

impl HeightTrackerMerger {
    pub fn new() -> HeightTrackerMerger {
        HeightTrackerMerger {
            heights: HashMap::new()
        }
    }

    pub fn merge(&mut self, other: &HeightTracker) {
        for (name,more_height) in other.heights.iter() {
            let height = self.heights.entry(name.clone()).or_insert(0);
            *height = (*height).max(*more_height);
        }
    }

    pub fn to_height_tracker(self) -> HeightTracker {
        let hash = HeightTracker::calc_hash(&self.heights);
        HeightTracker {
            hash,
            heights: Arc::new(self.heights)
        }
    }
}

#[derive(Clone)]
pub struct HeightTracker {
    hash: u64,
    heights: Arc<HashMap<AllotmentName,i64>>
}

impl PartialEq for HeightTracker {
    fn eq(&self, other: &Self) -> bool { self.hash == other.hash }
}

impl Eq for HeightTracker {}

impl Hash for HeightTracker {
    fn hash<H: Hasher>(&self, state: &mut H) { self.hash.hash(state); }
}

impl HeightTracker {
    const ROUND : f64 = 1000.;

    fn to_fixed(input: f64) -> i64 { (input*Self::ROUND).round() as i64 }
    fn from_fixed(input: i64) -> f64 { (input as f64)/Self::ROUND }

    fn calc_hash(input: &HashMap<AllotmentName,i64>) -> u64 {
        let mut names = input.keys().collect::<Vec<_>>();
        names.sort_by_cached_key(|name| name.hash_value());
        let mut state = DefaultHasher::new();
        for name in names {
            name.hash_value().hash(&mut state);
            input.get(name).cloned().unwrap().hash(&mut state);
        }
        state.finish()
    }

    pub(crate) fn empty() -> HeightTracker {
        let empty = HashMap::new();
        let hash = Self::calc_hash(&empty);
        HeightTracker {
            hash,
            heights: Arc::new(empty)
        }
    }

    pub(crate) fn new(pieces: &HeightTrackerPieces, answer_index: &mut StaticAnswer) -> HeightTracker {
        #[cfg(debug_assertions)]
        if !pieces.built {
            error!("unbuilt tracker!");
            panic!();
        }
        let mut out = HashMap::new();
        for (name,height) in lock!(pieces.heights).iter() {
            out.insert(name.clone(),Self::to_fixed(height.get_used(answer_index)));
        }
        let hash = Self::calc_hash(&out);
        HeightTracker { hash, heights: Arc::new(out) }
    }

    fn get(&self, name: &AllotmentName) -> f64 {
        Self::from_fixed(self.heights.get(name).cloned().unwrap_or(0))
    }
}

#[cfg(debug_assertions)]
impl fmt::Debug for HeightTracker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = vec![];
        for (name,height) in &*self.heights {
            out.push(format!("{:?}: {}",name,Self::from_fixed(*height)));
        }
        write!(f,"heights: {}",out.join(", "))
    }
}

struct HeightValue {
    value: f64,
    setter: Vec<UnknownSetter<'static,StaticValue<f64>>>
}

const PRECISION : f64 = 10000.;

impl Hash for HeightValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ((self.value*PRECISION).round() as i64).hash(state);
    }
}

impl HeightValue {
    fn empty() -> HeightValue {
        HeightValue { value: 0., setter: vec![] }
    }

    fn from_tracker(value: &HeightTrackerValue, answer: &mut StaticAnswer) -> HeightValue {
        HeightValue {
            value: value.output.call(answer),
            setter: vec![value.setter.clone()]
        }
    }

    fn merge(&mut self, other: &HeightValue) {
        self.value = self.value.max(other.value);
        self.setter.extend(other.setter.iter().cloned());
    }

    fn set(&self, answer: &mut StaticAnswer) {
        for setter in &self.setter {
            setter.set(answer,constant(self.value));
        }
    }
}

#[cfg(debug_assertions)]
impl fmt::Debug for HeightValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HeightValue").field("value", &self.value).finish()
    }
}

struct HeightTrackerValue {
    input: StaticValue<f64>,
    output: StaticValue<f64>,
    setter: UnknownSetter<'static,StaticValue<f64>>
}

impl HeightTrackerValue {
    fn new(input: &StaticValue<f64>, independent_answer: &mut StaticAnswer) -> HeightTrackerValue {
        let (mut setter,output) = short_unknown_function_promise();
        setter.set(independent_answer,input.clone());
        HeightTrackerValue { input: input.clone(), output, setter }
    }

    fn set(&mut self, answer: &mut StaticAnswer, value: f64) {
        self.setter.set(answer,constant(value));
    }
}

pub struct HeightTrackerPieces2 {
    values: HashMap<AllotmentName,HeightTrackerValue>
}

impl HeightTrackerPieces2 {
    pub(crate) fn new() -> HeightTrackerPieces2 {
        HeightTrackerPieces2 {
            values: HashMap::new()
        }
    }

    pub(crate) fn add(&mut self, name: &AllotmentName, input: &StaticValue<f64>, independent_answer: &mut StaticAnswer) -> StaticValue<f64> {
        if !self.values.contains_key(name) {
            self.values.insert(name.clone(),HeightTrackerValue::new(input,independent_answer));
        }
        self.values.get(name).unwrap().output.clone()
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct HeightTracker2Values {
    values: HashMap<AllotmentName,HeightValue>
}

impl Hash for HeightTracker2Values {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut names = self.values.keys().collect::<Vec<_>>();
        names.sort();
        for name in names {
            name.hash(state);
            self.values[name].hash(state);
        }
    }
}

impl HeightTracker2Values {
    pub fn empty() -> HeightTracker2Values {
        HeightTracker2Values {
            values: HashMap::new()
        }
    }

    pub fn new(pieces: &HeightTrackerPieces2, answer: &mut StaticAnswer) -> HeightTracker2Values {
        let mut values = HashMap::new();
        for (name,value) in &pieces.values {
            values.insert(name.clone(),HeightValue::from_tracker(value,answer));
        }
        HeightTracker2Values {
            values
        }
    }

    pub fn merge(&mut self, other: &HeightTracker2Values) {
        for (name,new_value) in &other.values {
            let value = self.values.entry(name.clone()).or_insert(HeightValue::empty());
            value.merge(new_value);
        }
    }

    pub fn set_answer(&self, answer: &mut StaticAnswer) {
        for value in self.values.values() {
            value.set(answer);
        }
    }
}
