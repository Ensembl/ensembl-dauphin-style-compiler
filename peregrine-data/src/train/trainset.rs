use std::sync::{Arc, Mutex};
use peregrine_toolkit::log;
use peregrine_toolkit::puzzle::AnswerAllocator;
use peregrine_toolkit::sync::blocker::{Blocker};
use peregrine_toolkit::sync::needed::Needed;
use crate::{CarriageExtent, CarriageSpeed, ShapeStore, PeregrineCoreBase};
use crate::api::MessageSender;
use crate::core::{Scale, Viewport};
use super::graphics::Graphics;
use super::railwaydatatasks::RailwayDataTasks;
use super::railwaydependents::RailwayDependents;
use super::switcher::{SwitcherExtent, SwitcherObject, SwitcherManager, Switcher};
use super::train::{ Train };
use super::trainextent::TrainExtent;
use crate::util::message::DataMessage;

struct TrainSetManager {
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
    type Type = Train;
    type Extent = TrainExtent;
    type Error = DataMessage;

    fn create(&mut self, extent: &Self::Extent) -> Result<Self::Type,Self::Error> {
        log!("create train: {:?}",extent);
        let mut train = Train::new(&self.graphics,&self.ping_needed,&self.answer_allocator,extent,&self.carriage_loader,&self.messages)?;
        if let Some(viewport) = &self.viewport {
            train.set_position(viewport);
        }
        Ok(train)
    }

    fn busy(&self, yn: bool) { self.dependents.busy(yn) }
}

impl SwitcherExtent for TrainExtent {
    type Type = Train;
    type Extent = TrainExtent;

    fn to_milestone(&self) -> Self::Extent {
        let scale = self.scale().clone().to_milestone();
        TrainExtent::new(self.layout(),&scale,self.pixel_size())
    }

    fn is_milestone_for(&self, what: &Self::Extent) -> bool {
        self.scale().is_milestone() &&
        self.layout() == what.layout() &&
        self.scale() == &what.scale().to_milestone()
    }
}

impl SwitcherObject for Train {
    type Extent = TrainExtent;
    type Type = Train;
    type Speed = CarriageSpeed;

    fn extent(&self) -> Self::Extent { self.extent().clone() }
    fn half_ready(&self) -> bool { self.train_half_ready() }
    fn ready(&self) -> bool { self.train_ready() }
    fn broken(&self) -> bool { self.train_broken() }
    fn live(&mut self, speed: &Self::Speed) { self.set_active(speed.clone()) }
    fn dead(&mut self) { self.set_inactive() }

    fn speed(&self, source: Option<&Self::Type>) -> Self::Speed {
        source.as_ref().map(|x| x.speed_limit(&self)).unwrap_or(CarriageSpeed::Quick)
    }
}

pub struct TrainSet(Switcher<TrainSetManager,TrainExtent,Train,DataMessage>);

impl TrainSet {
    pub(super) fn new(base: &PeregrineCoreBase, result_store: &ShapeStore, visual_blocker: &Blocker, ping_needed: &Needed, carriage_loader: &RailwayDataTasks) -> TrainSet {
        let manager = TrainSetManager {
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
        let extent = TrainExtent::new(viewport.layout()?,&best_scale,viewport.pixel_size()?);
        /* allow change of trains */
        self.0.set_target(&extent)?;
        /* set position in anticipator */
        if let Some(train) = self.0.quiescent() {
            let central_index = train.extent().scale().carriage(viewport.position()?);
            let central_carriage = CarriageExtent::new(&train.extent(),central_index);
            self.0.manager().dependents.position_was_updated(&central_carriage);
        }
        /* set position in all current trains (data) */
        let viewport_stick = viewport.layout()?.stick();
        self.0.each_mut(&|train| {
            if viewport_stick == train.extent().layout().stick() {
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
        self.0.force_no_match()
    }
}
