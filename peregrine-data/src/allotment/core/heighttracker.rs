use std::{collections::{HashMap, hash_map::{DefaultHasher}}, sync::Arc, fmt, hash::{Hash, Hasher}};

use peregrine_toolkit::{puzzle::{PuzzleValueHolder, PuzzleBuilder, FoldValue, PuzzleSolution, ClonablePuzzleValue, PuzzlePiece}, error, log};

use crate::allotment::{style::allotmentname::AllotmentName};

struct HeightTrackerEntry {
    extra: PuzzlePiece<f64>,
    used: FoldValue<f64>
}

impl HeightTrackerEntry {
    fn add(&mut self, height: &PuzzleValueHolder<f64>) {
        self.used.add(height);
    }

    fn build(&mut self) {
        self.used.build();
    }

    fn get_used(&self, solution: &PuzzleSolution) -> f64 {
        self.used.get().get_clone(solution)
    }

    fn set_full_height(&self, solution: &mut PuzzleSolution, full_height: f64) {
        self.extra.set_answer(solution,full_height);
    }

    fn get_piece(&self) -> &PuzzlePiece<f64> {
        &self.extra
    }
}

pub struct HeightTrackerPieces {
    puzzle: PuzzleBuilder,
    heights: HashMap<AllotmentName,HeightTrackerEntry>,

    #[cfg(debug_assertions)]
    built: bool,
}

impl HeightTrackerPieces {
    pub(crate) fn new(puzzle: &PuzzleBuilder) -> HeightTrackerPieces {
        HeightTrackerPieces {
            puzzle: puzzle.clone(),
            heights: HashMap::new(),
            #[cfg(debug_assertions)]
            built: false
        }
    }

    fn ensure_entry(&mut self, name: &AllotmentName) -> &mut HeightTrackerEntry {
        let puzzle = self.puzzle.clone();
        self.heights.entry(name.clone()).or_insert_with( || {
            let mut output = puzzle.new_piece();
            #[cfg(debug_assertions)]
            output.set_name("height tracker in");
            let mut extra = puzzle.new_piece_default(0.); // XXX no default
            #[cfg(debug_assertions)]
            extra.set_name("height tracker out");
            HeightTrackerEntry {  
                extra,
                used: FoldValue::new(output,|a,b| { f64::max(a,b) })
            }
        })
    }

    pub(crate) fn add(&mut self, name: &AllotmentName, height: &PuzzleValueHolder<f64>) {
        self.ensure_entry(name).add(height);
    }

    pub(crate) fn build(&mut self) {
        #[cfg(debug_assertions)]
        { self.built = true; }
        for values in self.heights.values_mut() {
            values.build();
        }
    }

    pub(crate) fn get_piece(&mut self, name: &AllotmentName) -> &PuzzlePiece<f64> {
        self.ensure_entry(name).get_piece()
    }

    /* must be called pre-solve */
    pub(crate) fn set_extra_height(&self, solution: &mut PuzzleSolution, heights: &HeightTracker) {
        for (name,entry) in &self.heights {
            entry.set_full_height(solution,heights.get(name));
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

    pub(crate) fn new(pieces: &HeightTrackerPieces, solution: &PuzzleSolution) -> HeightTracker {
        #[cfg(debug_assertions)]
        if !pieces.built {
            error!("unbuilt tracker!");
            panic!();
        }
        let mut out = HashMap::new();
        for (name,height) in pieces.heights.iter() {
            out.insert(name.clone(),Self::to_fixed(height.get_used(solution)));
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
