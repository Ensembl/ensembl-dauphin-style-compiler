use std::{sync::{Arc, Mutex}, collections::{HashMap, hash_map::DefaultHasher, HashSet}, fmt, hash::{Hash, Hasher}};
use peregrine_toolkit::{puzzle::{StaticAnswer, AnswerAllocator}, lock };
use crate::{allotment::{globals::{heighttracker::{LocalHeightTrackerBuilder, LocalHeightTracker, GlobalHeightTracker, GlobalHeightTrackerBuilder}, playingfield::{LocalPlayingFieldBuilder, LocalPlayingField, GlobalPlayingField, GlobalPlayingFieldBuilder}, aligner::{LocalAlignerBuilder, LocalAligner, GlobalAligner, GlobalAlignerBuilder}, allotmentmetadata::{LocalAllotmentMetadataBuilder, LocalAllotmentMetadata, GlobalAllotmentMetadata, GlobalAllotmentMetadataBuilder}, bumping::{LocalBumpBuilder, GlobalBump, GlobalBumpBuilder, LocalBump}, trainpersistent::TrainPersistent}}};

use lazy_static::lazy_static;
use identitynumber::identitynumber;

#[cfg(debug_trains)]
use peregrine_toolkit::{debug_log};

/* Every carriage manipulates in a CarriageTrainStateRequest during creation (during build). This specifies the
 * requirements which a Carriage has of the train. 
 */

pub struct CarriageTrainStateRequest {
    height_tracker: LocalHeightTrackerBuilder,
    playing_field: LocalPlayingFieldBuilder,
    aligner: LocalAlignerBuilder,
    metadata: LocalAllotmentMetadataBuilder,
    bumper: LocalBumpBuilder
}

impl CarriageTrainStateRequest {
    pub fn new() -> CarriageTrainStateRequest {
        CarriageTrainStateRequest {
            height_tracker: LocalHeightTrackerBuilder::new(),
            playing_field: LocalPlayingFieldBuilder::new(),
            aligner: LocalAlignerBuilder::new(),
            metadata: LocalAllotmentMetadataBuilder::new(),
            bumper: LocalBumpBuilder::new()
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

    pub fn bump(&self) -> &LocalBumpBuilder { &self.bumper }
    pub fn bump_mut(&mut self) -> &mut LocalBumpBuilder { &mut self.bumper }
}

#[derive(Clone)]
pub struct CarriageTrainStateSpec {
    height_values: Arc<LocalHeightTracker>,
    playing_field: Arc<LocalPlayingField>,
    aligner: Arc<LocalAligner>,
    metadata: Arc<LocalAllotmentMetadata>,
    bump: Arc<LocalBump>
}

impl CarriageTrainStateSpec {
    pub fn new(request: &CarriageTrainStateRequest) -> CarriageTrainStateSpec {
        let height_tracker = LocalHeightTracker::new(request.height_tracker());
        let playing_field = LocalPlayingField::new(request.playing_field());
        let aligner = LocalAligner::new(request.aligner());
        let metadata = LocalAllotmentMetadata::new(request.metadata());
        let bump = LocalBump::new(request.bump());
        CarriageTrainStateSpec {
            height_values: Arc::new(height_tracker),
            playing_field: Arc::new(playing_field),
            aligner: Arc::new(aligner),
            metadata: Arc::new(metadata),
            bump: Arc::new(bump),
        }
    }
}

identitynumber!(IDS);

#[derive(Clone)]
pub struct TrainState3 {
    indexes: Arc<Mutex<HashSet<u64>>>,
    height_tracker: Arc<GlobalHeightTracker>,
    playing_field: Arc<GlobalPlayingField>,
    metadata: Arc<GlobalAllotmentMetadata>,
    aligner: Arc<GlobalAligner>,
    bump: Arc<GlobalBump>,
    answer: Arc<Mutex<StaticAnswer>>,
    hash: u64,
    serial: u64
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
        write!(f,"state({})",self.serial)
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

    fn calc_bump(spec: &TrainStateSpec, answer: &mut StaticAnswer, persistent: &Arc<Mutex<TrainPersistent>>) -> GlobalBump {
        let mut builder = GlobalBumpBuilder::new();
        for carriage_spec in spec.specs.values() {
            carriage_spec.bump.add(&mut builder);
        }        
        GlobalBump::new(builder,answer,persistent)
    }

    fn calc_hash(&mut self) {
        let mut hasher = DefaultHasher::new();
        self.height_tracker.hash(&mut hasher);
        self.playing_field.hash(&mut hasher);
        self.aligner.hash(&mut hasher);
        self.metadata.hash(&mut hasher);
        self.bump.hash(&mut hasher);
        self.hash = hasher.finish();
    }

    pub(crate) fn add(&self, index: u64, state: &CarriageTrainStateSpec) {
        let mut indexes = lock!(self.indexes);
        if indexes.contains(&index) { return; }
        indexes.insert(index);
        drop(indexes);
        let mut answer = lock!(self.answer);
        self.bump.add(&state.bump,&mut answer);
        self.height_tracker.add(&state.height_values,&mut answer);
        self.playing_field.add(&state.playing_field,&mut answer);
        self.aligner.add(&state.aligner,&mut answer);
    }

    // TODO new add method to add to existing trait by populating answers

    fn new(spec: &TrainStateSpec, mut answer: StaticAnswer, persistent: &Arc<Mutex<TrainPersistent>>) -> TrainState3 {
        /* The order of these are important. Their funcs can only depend on preceding funcs.
         * eg. heights depend on bumps, so bumps must be first. Everything else depends on
         * heights, so these two must be before the others.
         */
        let bump = Arc::new(Self::calc_bump(spec,&mut answer,persistent));
        let height_tracker = Arc::new(Self::calc_heights(spec,&mut answer));
        let playing_field = Arc::new(Self::calc_playing_field(spec,&mut answer));
        let aligner = Arc::new(Self::calc_aligner(spec,&mut answer));
        let metadata = Arc::new(Self::calc_metadata(spec,&mut answer));
        let mut out = TrainState3 {
            indexes: Arc::new(Mutex::new(spec.specs.keys().cloned().collect::<HashSet<_>>())),
            height_tracker, playing_field, aligner, metadata, bump,
            answer: Arc::new(Mutex::new(answer)), hash: 0,
            serial: IDS.next()
        };
        out.calc_hash();
        out
    }

    #[cfg(debug_assertions)]
    #[allow(unused)]
    pub(crate) fn hash(&self) -> u64 { self.hash }

    pub(crate) fn answer(&self) -> Arc<Mutex<StaticAnswer>> { self.answer.clone() }
    pub(crate) fn playing_field(&self) -> &GlobalPlayingField { &self.playing_field }
    pub(crate) fn metadata(&self) -> &GlobalAllotmentMetadata { &self.metadata }
}

pub struct TrainStateSpec {
    answer_allocator: Arc<Mutex<AnswerAllocator>>,
    specs: HashMap<u64,CarriageTrainStateSpec>,
    cached_train_state: Mutex<Option<TrainState3>>,
    persistent: Arc<Mutex<TrainPersistent>>,
    old_states: Mutex<HashMap<u64,TrainState3>>
}

impl TrainStateSpec {
    pub fn new(answer_allocator: &Arc<Mutex<AnswerAllocator>>) -> TrainStateSpec {
        TrainStateSpec {
            answer_allocator: answer_allocator.clone(),
            specs: HashMap::new(),
            cached_train_state: Mutex::new(None),
            persistent: Arc::new(Mutex::new(TrainPersistent::new())),
            old_states: Mutex::new(HashMap::new()),
        }
    }

    pub fn spec(&self) -> TrainState3 {
        let mut state = lock!(self.cached_train_state);
        if state.is_none() {
            let answer = lock!(self.answer_allocator).get();
            let new_state = TrainState3::new(self,answer,&self.persistent);
            let mut old = lock!(self.old_states);
            if let Some(existing_state) = old.get(&new_state.hash) {
                *state = Some(existing_state.clone());
            } else {
                #[cfg(debug_trains)] debug_log!("new state");
                old.insert(new_state.hash,new_state.clone());
                *state = Some(new_state);
            }
        }
        state.clone().unwrap()
    }

    pub(crate) fn add(&mut self, index: u64, spec: &CarriageTrainStateSpec) {
        self.specs.insert(index,spec.clone());
        *lock!(self.cached_train_state) = None;
    }

    pub(crate) fn remove(&mut self, index: u64) {
        self.specs.remove(&index);
        *lock!(self.cached_train_state) = None;
    }
}
