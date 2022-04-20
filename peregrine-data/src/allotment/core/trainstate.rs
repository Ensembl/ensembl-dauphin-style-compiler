use std::{sync::{Arc, Mutex}, collections::{HashMap, hash_map::DefaultHasher}, fmt, hash::{Hash, Hasher}};

use peregrine_toolkit::{puzzle::{StaticAnswer, AnswerAllocator}, lock, log, debug_log};

use crate::GlobalAllotmentMetadata;

use super::{playingfield::{LocalPlayingFieldBuilder, LocalPlayingField, GlobalPlayingFieldBuilder, GlobalPlayingField}, globalvalue::GlobalValueBuilder, heighttracker::{LocalHeightTrackerBuilder, LocalHeightTracker, GlobalHeightTracker, GlobalHeightTrackerBuilder}, aligner::{LocalAlignerBuilder, LocalAligner, GlobalAligner, GlobalAlignerBuilder}, allotmentmetadata::{LocalAllotmentMetadataBuilder, LocalAllotmentMetadata, GlobalAllotmentMetadataBuilder}};

/* Every carriage manipulates in a CarriageTrainStateRequest during creation (during build). This specifies the
 * requirements which a Carriage has of the train. 
 */

pub struct CarriageTrainStateRequest {
    height_tracker: LocalHeightTrackerBuilder,
    playing_field: LocalPlayingFieldBuilder,
    aligner: LocalAlignerBuilder,
    metadata: LocalAllotmentMetadataBuilder
}

impl CarriageTrainStateRequest {
    pub fn new() -> CarriageTrainStateRequest {
        CarriageTrainStateRequest {
            height_tracker: LocalHeightTrackerBuilder::new(),
            playing_field: LocalPlayingFieldBuilder::new(),
            aligner: LocalAlignerBuilder::new(),
            metadata: LocalAllotmentMetadataBuilder::new()
        }
    }

    pub fn playing_field(&self) -> &LocalPlayingFieldBuilder { &self.playing_field }
    pub fn playing_field_mut(&mut self) -> &mut LocalPlayingFieldBuilder { &mut self.playing_field }

    pub fn height_tracker(&self) -> &LocalHeightTrackerBuilder { &self.height_tracker }
    pub fn height_tracker_mut(&mut self) -> &mut LocalHeightTrackerBuilder { &mut self.height_tracker }

    pub fn aligner(&self) -> &LocalAlignerBuilder { &self.aligner }
    pub fn aligner_mut(&mut self) -> &mut LocalAlignerBuilder { &mut self.aligner }

    pub fn metadata(&self) -> &LocalAllotmentMetadataBuilder { &self.metadata }
    pub fn metadata_mut(&mut self) -> &mut LocalAllotmentMetadataBuilder { &mut self.metadata }
}

#[derive(Clone)]
pub struct CarriageTrainStateSpec {
    height_values: Arc<LocalHeightTracker>,
    playing_field: Arc<LocalPlayingField>,
    aligner: Arc<LocalAligner>,
    metadata: Arc<LocalAllotmentMetadata>
}

impl CarriageTrainStateSpec {
    pub fn new(request: &CarriageTrainStateRequest, independent_answer: &mut StaticAnswer) -> CarriageTrainStateSpec {
        let height_tracker = LocalHeightTracker::new(request.height_tracker(),independent_answer);
        let playing_field = LocalPlayingField::new(request.playing_field(),independent_answer);
        let aligner = LocalAligner::new(request.aligner(),independent_answer);
        let metadata = LocalAllotmentMetadata::new(request.metadata());
        CarriageTrainStateSpec {
            height_values: Arc::new(height_tracker),
            playing_field: Arc::new(playing_field),
            aligner: Arc::new(aligner),
            metadata: Arc::new(metadata)
        }
    }
}

#[derive(Clone)]
pub struct TrainState3 {
    height_tracker: Arc<GlobalHeightTracker>,
    playing_field: Arc<GlobalPlayingField>,
    metadata: Arc<GlobalAllotmentMetadata>,
    aligner: Arc<GlobalAligner>,
    answer: Arc<Mutex<StaticAnswer>>,
    hash: u64
}

impl PartialEq for TrainState3 {
    fn eq(&self, other: &Self) -> bool { self.hash == other.hash }
}

impl Eq for TrainState3 {}

impl Hash for TrainState3 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

#[cfg(debug_assertions)]
impl fmt::Debug for TrainState3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrainState3").field("height_tracker", &self.height_tracker).finish()
    }
}

impl TrainState3 {
    fn calc_heights(spec: &TrainStateSpec, answer: &mut StaticAnswer) -> GlobalHeightTracker {
        let mut builder = GlobalHeightTrackerBuilder::new();
        for carriage_spec in spec.specs.values() {
            carriage_spec.height_values.add(&mut builder);
        }
        GlobalHeightTracker::new(builder,answer)
    }

    fn calc_playing_field(spec: &TrainStateSpec, answer: &mut StaticAnswer) -> GlobalPlayingField {
        let mut builder = GlobalPlayingFieldBuilder::new();
        for carriage_spec in spec.specs.values() {
            carriage_spec.playing_field.add(&mut builder);
        }
        GlobalPlayingField::new(builder,answer)
    }

    fn calc_aligner(spec: &TrainStateSpec, answer: &mut StaticAnswer) -> GlobalAligner {
        let mut builder = GlobalAlignerBuilder::new();
        for carriage_spec in spec.specs.values() {
            carriage_spec.aligner.add(&mut builder);
        }
        GlobalAligner::new(builder,answer)
    }

    fn calc_metadata(spec: &TrainStateSpec, answer: &mut StaticAnswer) -> GlobalAllotmentMetadata {
        let mut builder = GlobalAllotmentMetadataBuilder::new();
        for carriage_spec in spec.specs.values() {
            carriage_spec.metadata.add(&mut builder);
        }
        GlobalAllotmentMetadata::new(builder,answer)
    }

    fn calc_hash(&mut self) {
        let mut hasher = DefaultHasher::new();
        self.height_tracker.hash(&mut hasher);
        self.playing_field.hash(&mut hasher);
        self.aligner.hash(&mut hasher);
        self.metadata.hash(&mut hasher);
        self.hash = hasher.finish();
    }

    fn new(spec: &TrainStateSpec, mut answer: StaticAnswer) -> TrainState3 {
        let height_tracker = Arc::new(Self::calc_heights(spec,&mut answer));
        let playing_field = Arc::new(Self::calc_playing_field(spec,&mut answer));
        let aligner = Arc::new(Self::calc_aligner(spec,&mut answer));
        let metadata = Arc::new(Self::calc_metadata(spec,&mut answer));
        let mut out = TrainState3 {
            height_tracker, playing_field, aligner, metadata,
            answer: Arc::new(Mutex::new(answer)), hash: 0
        };
        out.calc_hash();
        out
    }

    pub(crate) fn answer(&self) -> Arc<Mutex<StaticAnswer>> { self.answer.clone() }
    pub(crate) fn playing_field(&self) -> &GlobalPlayingField { &self.playing_field }
    pub(crate) fn metadata(&self) -> &GlobalAllotmentMetadata { &self.metadata }
}

pub struct TrainStateSpec {
    answer_allocator: Arc<Mutex<AnswerAllocator>>,
    specs: HashMap<u64,CarriageTrainStateSpec>,
    cached_train_state: Mutex<Option<TrainState3>>
}

impl TrainStateSpec {
    pub fn new(answer_allocator: &Arc<Mutex<AnswerAllocator>>) -> TrainStateSpec {
        TrainStateSpec {
            answer_allocator: answer_allocator.clone(),
            specs: HashMap::new(),
            cached_train_state: Mutex::new(None)
        }
    }

    pub fn spec(&self) -> TrainState3 {
        let mut state = lock!(self.cached_train_state);
        if state.is_none() {
            let answer = lock!(self.answer_allocator).get();
            *state = Some(TrainState3::new(self,answer));
            debug_log!("new state: {:?}",*state);
        }
        state.clone().unwrap()
    }

    pub fn add(&mut self, index: u64, spec: &CarriageTrainStateSpec) {
        self.specs.insert(index,spec.clone());
        *lock!(self.cached_train_state) = None;
    }

    pub fn remove(&mut self, index: u64) {
        self.specs.remove(&index);
        *lock!(self.cached_train_state) = None;
    }
}
