use std::sync::{Arc, Mutex};
use peregrine_toolkit::{debug_log, lock};
use peregrine_toolkit::puzzle::AnswerAllocator;
use peregrine_toolkit_async::sync::blocker::{Blocker, LockoutBool};
use peregrine_toolkit_async::sync::needed::Needed;
use crate::shapeload::anticipate::Anticipate;
use crate::switch::trackconfiglist::TrainTrackConfigList;
use crate::train::core::switcher::{Switcher, SwitcherManager, SwitcherExtent, SwitcherObject};
use crate::train::graphics::Graphics;
use crate::train::model::trainextent::TrainExtent;
use crate::{CarriageSpeed, ShapeStore, PeregrineCoreBase, CarriageExtent, PeregrineApiQueue};
use crate::api::MessageSender;
use crate::core::{Scale, Viewport};
use crate::util::message::DataMessage;
use super::train::Train;

/* A railway gets input from three sources:
 * 1. on user request (from devices or API);
 * 2. on completion of data loading tasks;
 * 3. on completion of graphics preparation tasks.
 *
 * In response to these it starts graphics preparation tasks.
 * 
 * User-request inputs and graphics-preparation outputs are debounced per raf. Data-loading
 * inputs and graphics-preparation inputs are handled immediately.
 */

struct RailwayActions {
    api_queue: PeregrineApiQueue,
    current_epoch: u64,
    graphics: Graphics,
    answer_allocator: Arc<Mutex<AnswerAllocator>>,
    messages: MessageSender,
    busy: LockoutBool,
    anticipate: Anticipate,
    viewport: Option<Viewport>
}

impl RailwayActions {
    fn set_viewport(&mut self, viewport: &Viewport) {
        self.viewport = Some(viewport.clone());
    }
}

impl SwitcherManager for RailwayActions {
    type Type = Train;
    type Extent = SwitcherTrainExtent;
    type Error = DataMessage;

    fn create(&mut self, extent: &Self::Extent) -> Result<Self::Type,Self::Error> {
        #[cfg(debug_trains)] debug_log!("TRAIN create ({})",extent.extent.scale().get_index());
        let train_track_config_list = TrainTrackConfigList::new(&extent.extent.layout(),&extent.extent.scale());
        let mut train = Train::new(&self.api_queue,&extent.extent,&self.answer_allocator,&train_track_config_list,&self.graphics,&self.messages,self.current_epoch);
        if let Some(viewport) = &self.viewport {
            train.set_position(viewport);
        }
        Ok(train)
    }

    fn busy(&self, yn: bool) { self.busy.set(yn) }
}

#[derive(Clone,PartialEq,Eq)]
pub(crate) struct SwitcherTrainExtent {
    epoch: u64,
    extent: TrainExtent
}

impl SwitcherExtent for SwitcherTrainExtent {
    type Type = Train;
    type Extent = SwitcherTrainExtent;

    fn to_milestone(&self) -> Self::Extent {
        let scale = self.extent.scale().to_milestone();
        SwitcherTrainExtent {
            epoch: self.epoch,
            extent: TrainExtent::new(self.extent.layout(),&scale,self.extent.pixel_size())
        }
    }

    fn is_milestone_for(&self, what: &Self::Extent) -> bool {
        self.extent.scale().is_milestone() &&
        self.extent.layout() == what.extent.layout() &&
        self.extent.scale() == &what.extent.scale().to_milestone()
    }
}

impl SwitcherObject for Train {
    type Extent = SwitcherTrainExtent;
    type Type = Train;
    type Speed = CarriageSpeed;

    fn extent(&self) -> Self::Extent { 
        SwitcherTrainExtent {
            extent: self.train_extent().clone(),
            epoch: self.epoch()
        }
    }
    fn half_ready(&self) -> bool { self.train_half_ready() }
    fn ready(&self) -> bool { self.train_ready() }
    fn broken(&self) -> bool { self.train_broken() }
    fn live(&mut self, speed: &Self::Speed) -> bool {
        self.set_active(speed.clone());
        false
    }
    fn dead(&mut self) { self.mute(true) }

    fn speed(&self, source: Option<&Self::Type>) -> Self::Speed {
        return CarriageSpeed::Quick;
        if let Some(source) = source {
            if source.epoch() != self.epoch() { return CarriageSpeed::Slow; }
            source.speed_limit(&self)
        } else {
            CarriageSpeed::Quick
        }
    }
}

struct RailwayState(Switcher<RailwayActions,SwitcherTrainExtent,Train,DataMessage>);

impl RailwayState {
    pub(crate) fn new(base: &PeregrineCoreBase, result_store: &ShapeStore, visual_blocker: &Blocker) -> RailwayState {
        let manager = RailwayActions {
            api_queue: base.queue.clone(),
            current_epoch: 0,
            graphics: base.graphics.clone(),
            answer_allocator: base.answer_allocator.clone(),
            messages: base.messages.clone(),
            anticipate: Anticipate::new(base,result_store),
            busy: LockoutBool::new(visual_blocker),
            viewport: None
        };
        RailwayState(Switcher::new(manager))
    }

    pub(super) fn set_position(&mut self, viewport: &Viewport) -> Result<(),DataMessage> {
        if !viewport.ready() { return Ok(()); }
        /* calculate best train */
        let best_scale = Scale::new_bp_per_screen(viewport.bp_per_screen()?);
        let extent = SwitcherTrainExtent {
            epoch: self.0.manager().current_epoch,
            extent: TrainExtent::new(viewport.layout()?,&best_scale,viewport.pixel_size()?)
        };
        /* allow change of trains */
        self.0.set_target(&extent)?;
        /* set position in anticipator */
        if let Some(train) = self.0.quiescent() {
            let central_index = train.extent().extent.scale().carriage(viewport.position()?);
            let central_carriage = CarriageExtent::new(&train.train_extent(),central_index);
            self.0.manager().anticipate.anticipate(&central_carriage);
        }
        /* set position in all current trains (data) */
        let viewport_stick = viewport.layout()?.stick();
        self.0.each_mut(&|train| {
            if viewport_stick == train.extent().extent.layout().stick() {
                train.set_position(viewport); // XXX error handling
            }
        });
        /* set position for future trains (data) */
        self.0.manager_mut().set_viewport(viewport);
        /* set position in all trains (graphics) */
        self.0.manager().graphics.notify_viewport(viewport);
        Ok(())
    }

    pub(super) fn transition_complete(&mut self) {
        self.0.manager_mut().graphics.transition_complete();
        self.0.live_done();
    }

    pub(super) fn ping(&mut self) {
        self.0.ping();
        self.0.each_mut( &|c| { c.ping() });
    }

    pub(super) fn set_sketchy(&mut self, yn: bool) -> Result<(),DataMessage> {
        self.0.set_sketchy(yn)
    }

    pub(super) fn invalidate(&mut self) -> Result<(),DataMessage> {
        self.0.manager_mut().current_epoch += 1;
        if let Some(mut target) = self.0.get_target().cloned() {
            target.epoch = self.0.manager().current_epoch;
            self.0.set_target(&target)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Railway(Arc<Mutex<RailwayState>>);

impl Railway {
    pub(crate) fn new(base: &PeregrineCoreBase, result_store: &ShapeStore, visual_blocker: &Blocker) -> Railway {
        Railway(Arc::new(Mutex::new(RailwayState::new(base,result_store,visual_blocker))))
    }

    pub(crate) fn ping(&self) {
        lock!(self.0).ping();
    }

    pub(crate) fn set(&self, viewport: &Viewport) -> Result<(),DataMessage> {
        lock!(self.0).set_position(viewport)
    }

    pub(crate) fn transition_complete(&self) {
        lock!(self.0).transition_complete()
    }

    pub(crate) fn set_sketchy(&self, yn: bool) -> Result<(),DataMessage> {
        lock!(self.0).set_sketchy(yn)
    }

    pub(crate) fn invalidate(&self) -> Result<(),DataMessage> {
        lock!(self.0).invalidate()
    }
}
