use std::sync::{Arc, Mutex};
use peregrine_toolkit::{debug_log};
use peregrine_toolkit::puzzle::AnswerAllocator;
use peregrine_toolkit::sync::blocker::{Blocker};
use peregrine_toolkit::sync::needed::Needed;
use crate::{CarriageSpeed, ShapeStore, PeregrineCoreBase};
use crate::api::MessageSender;
use crate::core::{Scale, Viewport};
use super::carriageextent::CarriageExtent;
use super::graphics::Graphics;
use super::railwaydatatasks::RailwayDataTasks;
use super::railwaydependents::RailwayDependents;
use super::switcher::{SwitcherExtent, SwitcherObject, SwitcherManager, Switcher};
use super::train::{ Train };
use super::trainextent::TrainExtent;
use crate::util::message::DataMessage;

struct TrainSetManager {
    current_epoch: u64,
    ping_needed: Needed,
    graphics: Graphics,
    answer_allocator: Arc<Mutex<AnswerAllocator>>,
    messages: MessageSender,
    carriage_loader: RailwayDataTasks,
    dependents: RailwayDependents,
    viewport: Option<Viewport>
}

impl TrainSetManager {
    fn set_viewport(&mut self, viewport: &Viewport) {
        self.viewport = Some(viewport.clone());
    }
}

impl SwitcherManager for TrainSetManager {
    type Type = SwitcherTrain;
    type Extent = SwitcherTrainExtent;
    type Error = DataMessage;

    fn create(&mut self, extent: &Self::Extent) -> Result<Self::Type,Self::Error> {
        debug_log!("create train: {:?}",extent.extent);
        let mut train = Train::new(&self.graphics,&self.ping_needed,&self.answer_allocator,&extent.extent,&self.carriage_loader,&self.messages)?;
        if let Some(viewport) = &self.viewport {
            train.set_position(viewport);
        }
        Ok(SwitcherTrain {
            train,
            epoch: self.current_epoch
        })
    }

    fn busy(&self, yn: bool) { self.dependents.busy(yn) }
}

#[derive(Clone,PartialEq,Eq)]
struct SwitcherTrainExtent {
    epoch: u64,
    extent: TrainExtent
}

impl SwitcherExtent for SwitcherTrainExtent {
    type Type = SwitcherTrain;
    type Extent = SwitcherTrainExtent;

    fn to_milestone(&self) -> Self::Extent {
        let scale = self.extent.scale().clone().to_milestone();
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

struct SwitcherTrain {
    epoch: u64,
    train: Train
}

impl SwitcherObject for SwitcherTrain {
    type Extent = SwitcherTrainExtent;
    type Type = SwitcherTrain;
    type Speed = CarriageSpeed;

    fn extent(&self) -> Self::Extent { 
        SwitcherTrainExtent {
            extent: self.train.extent().clone(),
            epoch: self.epoch
        }
    }
    fn half_ready(&self) -> bool { self.train.train_half_ready() }
    fn ready(&self) -> bool { self.train.train_ready() }
    fn broken(&self) -> bool { self.train.train_broken() }
    fn live(&mut self, speed: &Self::Speed) { self.train.set_active(speed.clone()) }
    fn dead(&mut self) { self.train.set_inactive() }

    fn speed(&self, source: Option<&Self::Type>) -> Self::Speed {
        return CarriageSpeed::Quick;
        if let Some(source) = source {
            if source.epoch != self.epoch { return CarriageSpeed::Slow; }
            source.train.speed_limit(&self.train)
        } else {
            CarriageSpeed::Quick
        }
    }
}

pub struct TrainSet(Switcher<TrainSetManager,SwitcherTrainExtent,SwitcherTrain,DataMessage>);

impl TrainSet {
    pub(super) fn new(base: &PeregrineCoreBase, result_store: &ShapeStore, visual_blocker: &Blocker, ping_needed: &Needed, carriage_loader: &RailwayDataTasks) -> TrainSet {
        let manager = TrainSetManager {
            current_epoch: 0,
            ping_needed: ping_needed.clone(),
            graphics: base.graphics.clone(),
            answer_allocator: base.answer_allocator.clone(),
            messages: base.messages.clone(),
            carriage_loader: carriage_loader.clone(),
            dependents:  RailwayDependents::new(base,result_store,visual_blocker),
            viewport: None
        };
        TrainSet(Switcher::new(manager))
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
            let central_carriage = CarriageExtent::new(&train.train.extent(),central_index);
            self.0.manager().dependents.position_was_updated(&central_carriage);
        }
        /* set position in all current trains (data) */
        let viewport_stick = viewport.layout()?.stick();
        self.0.each_mut(&|train| {
            if viewport_stick == train.extent().extent.layout().stick() {
                train.train.set_position(viewport); // XXX error handling
            }
        });
        /* set position for future trains (data) */
        self.0.manager_mut().set_viewport(viewport);
        /* set position in all trains (graphics) */
        self.0.manager().graphics.notify_viewport(viewport);
        Ok(())
    }

    pub(super) fn transition_complete(&mut self) {
        self.0.live_done();
    }

    pub(super) fn ping(&mut self) {
        self.0.ping();
        self.0.each_mut( &|c| { c.train.ping() });
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
