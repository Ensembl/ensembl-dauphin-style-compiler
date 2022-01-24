use peregrine_toolkit::sync::blocker::{Blocker};
use peregrine_toolkit::sync::needed::Needed;
use crate::{CarriageExtent, CarriageSpeed, LaneStore, PeregrineCoreBase};
use crate::api::MessageSender;
use crate::core::{Layout, Scale, Viewport};
use super::railwaydependents::RailwayDependents;
use super::train::{ Train };
use super::carriage::{Carriage, CarriageSerialSource};
use super::railwayevent::RailwayEvents;
use super::trainextent::TrainExtent;
use crate::util::message::DataMessage;

/* current: train currently being displayed, if any. During transition, the outgoing train.
 * future: incoming train during transition.
 * wanted: train to be displayed when complete and when transitions are done.
 */

pub struct TrainSet {
    try_lifecycle: Needed,
    serial_source: CarriageSerialSource,
    current: Option<Train>,
    future: Option<Train>,
    wanted: Option<Train>,
    target: Option<TrainExtent>,
    target_validity_counter: u64,
    next_train_serial: u64,
    messages: MessageSender,
    dependents: RailwayDependents,
    viewport: Option<Viewport>,
    sketchy: bool,
    validity_counter: u64
}

impl TrainSet {
    pub(super) fn new(base: &PeregrineCoreBase, result_store: &LaneStore, visual_blocker: &Blocker, try_lifecycle: &Needed) -> TrainSet {
        let serial_source = CarriageSerialSource::new();
        TrainSet {
            try_lifecycle: try_lifecycle.clone(),
            current: None,
            future: None,
            wanted: None,
            target: None,
            next_train_serial: 0,
            messages: base.messages.clone(),
            dependents: RailwayDependents::new(base,result_store,&serial_source,visual_blocker,try_lifecycle),
            serial_source,
            viewport: None,
            sketchy: false,
            validity_counter: 1,
            target_validity_counter: 0
        }
    }

    fn quiescent_target(&self) -> Option<&Train> {
        /* The quiescent train is the train which, barring this and any future changes will ultimately be displayed. */
        if let Some(wanted) = &self.wanted {
            Some(wanted)
        } else if let Some(future) = &self.future {
            Some(future)
        } else if let Some(current) = &self.current {
            Some(current)
        } else {
            None
        }
    }

    fn each_current_train<X,F>(&self, state: &mut X, cb: &F) where F: Fn(&mut X,&Train) {
        if let Some(wanted) = &self.wanted { cb(state,wanted); }
        if let Some(future) = &self.future { cb(state,future); }
        if let Some(current) = &self.current { cb(state,current); }
    }

    fn each_current_carriage<X,F>(&self, state: &mut X, cb: &F) where F: Fn(&mut X,&Carriage) {
        self.each_current_train(state,&|state,train| train.each_current_carriage(state,cb));
    }

    fn all_current_carriages(&self) -> Vec<Carriage> {
        let mut out = vec![];
        self.each_current_carriage(&mut out, &|out,carriage| {
            out.push(carriage.clone())
        });
        out
    }

    fn try_advance_wanted_to_future(&mut self, events: &mut RailwayEvents) {
        let desperate = self.future.is_none() && self.current.is_none();
        let train_good_enough = self.wanted.as_ref().map(|x| {
            let train_ready_enough = x.train_ready() || (desperate && x.train_half_ready());
            train_ready_enough && !x.train_broken()
        }).unwrap_or(false);
        if train_good_enough && self.future.is_none() {
            if let Some(mut wanted) = self.wanted.take() {
                let speed = self.current.as_ref().map(|x| x.speed_limit(&wanted)).unwrap_or(CarriageSpeed::Quick);
                wanted.set_active(events,speed);
                let viewport = wanted.viewport();
                self.future = Some(wanted);
                self.dependents.carriages_loaded(self.quiescent_target(),&self.all_current_carriages(),events);
                self.draw_notify_viewport(events);
                self.try_new_wanted(events,&viewport);
            }
        }
    }

    fn draw_notify_viewport(&self, events: &mut RailwayEvents) {
        if let Some(train) = self.future.as_ref().or_else(|| self.current.as_ref()) {
            events.draw_notify_viewport(&train.viewport(),false);
        }
    }

    fn wanted_is_relevant_milestone(&self, old_layout: &Layout) -> bool {
        if self.wanted.is_none() {
            /* nothing in wanted */
            return false;
        }
        let wanted = self.wanted.as_ref().unwrap();
        if !wanted.extent().scale().is_milestone(){
            /* wanted is not a milestone */
            return false;
        }
        if wanted.extent().layout() != old_layout {
            /* wanted is irrelevant milestone */
            return false;
        }
        true
    }

    fn try_set_target(&mut self, viewport: &Viewport) -> Result<(),DataMessage> {
        let target_has_bad_validity_counter = self.validity_counter != self.target_validity_counter;
        let best_scale = Scale::new_bp_per_screen(viewport.bp_per_screen()?);
        let extent = TrainExtent::new(viewport.layout()?,&best_scale);
        if let Some(quiescent) = self.quiescent_target() {
            if quiescent.extent() == extent && !target_has_bad_validity_counter {
                return Ok(()); //no need for a target, we're heading to the right place
            }
        }
        let extent = TrainExtent::new(viewport.layout()?,&best_scale);
        self.target = Some(extent);
        self.target_validity_counter = self.validity_counter;
        Ok(())
    }

    /* Never discard a milestone (with our layout). If discarding anything, convert to relevant milestone.
     * Prevents thrashing of scales when busy.
     */
    fn try_new_wanted(&mut self, events: &mut RailwayEvents, viewport: &Viewport) -> Result<(),DataMessage> {
        if self.target.is_none() { return Ok(()); }
        let target = self.target.as_ref().unwrap();
        let target_validity_matches_quiescent = if let Some(quiescent) = self.quiescent_target().cloned() {
            /* if we are now heading exactly for the target, drop it for future calls */
            quiescent.validity_counter() == self.target_validity_counter
        } else {
            true
        };
        /* it would be best if we were at a new target, but how busy are we? */
        if self.wanted_is_relevant_milestone(target.layout()) { return Ok(()); } // don't evict milestone  
        /* where do we want to head? */
        let mut scale = target.scale().clone();
        if self.wanted.is_some() || self.sketchy {
            scale = scale.to_milestone();
        }
        let extent = TrainExtent::new(target.layout(),&scale);
        /* drop old wanted and make milestone, if necessary */      
        if let Some(mut wanted) = self.wanted.take() {
            wanted.discard(events);
        }
        if let Some(quiescent) = self.quiescent_target().cloned() {
            /* if we are now heading exactly for the target, drop it for future calls */
            if &extent == target && target_validity_matches_quiescent {
                self.target.take();
            }
            /* is this where we were heading anyway? */
            if quiescent.extent() == extent && target_validity_matches_quiescent {
                return Ok(());
            }    
        }
        /* do it */
        self.next_train_serial +=1;
        let wanted = Train::new(&self.try_lifecycle,self.next_train_serial,&extent,events,viewport,&self.messages,&self.serial_source,self.target_validity_counter)?;
        events.draw_create_train(&wanted);
        self.wanted = Some(wanted);
        Ok(())
    }

    fn min_quiescent_validity(&self) -> u64 {
        if self.target.is_some() {
            self.target_validity_counter
        } else if let Some(quiescent) = self.quiescent_target() {
            quiescent.validity_counter()
        } else {
            0
        }
    }

    pub(super) fn set_position(&mut self, events: &mut RailwayEvents, viewport: &Viewport) -> Result<(),DataMessage> {
        if !viewport.ready() { return Ok(()); }
        self.viewport = Some(viewport.clone());
        /* maybe we need to change the wanted train? */
        self.try_set_target(viewport)?;
        self.try_new_wanted(events,viewport)?;
        /* dependents need to know we moved */
        if let Some(train) = self.quiescent_target() {
            let central_index = train.extent().scale().carriage(viewport.position()?);
            let central_carriage = CarriageExtent::new(&train.extent(),central_index);
            self.dependents.position_was_updated(&central_carriage);
        }
        /* All the trains get the new position */
        let min_quiescent_validity = self.min_quiescent_validity();
        let viewport_stick = viewport.layout()?.stick();
        self.each_current_train(events,&|events,train| {
            if viewport_stick == train.extent().layout().stick() && train.validity_counter() >= min_quiescent_validity {
                train.set_position(&mut events.clone(),viewport); // XXX error handling
            }
        });
        /* check if any progress can be made */
        self.try_advance_wanted_to_future(events);
        /* tell dependents */
        self.draw_notify_viewport(events);
        Ok(())
    }

    fn reset_position(&mut self, events: &mut RailwayEvents) -> Result<(),DataMessage> {
        if !self.viewport.is_some() { return Ok(()); }
        self.set_position(events,&self.viewport.as_ref().cloned().unwrap())
    }

    pub(super) fn transition_complete(&mut self, events: &mut RailwayEvents) {
        /* retire current and make future current */
        if let Some(mut current) = self.current.take() {
            current.discard(events);
            events.draw_drop_train(&current);
        }
        self.current = self.future.take();
        /* now future is free, maybe wanted can go there? */
        self.try_advance_wanted_to_future(events);
        /* stuff may have happened above, tell dependents */
        self.dependents.carriages_loaded(self.quiescent_target(),&self.all_current_carriages(),events);
    }

    pub(super) fn move_and_lifecycle_trains(&mut self) -> RailwayEvents {
        let mut events = RailwayEvents::new();
        if let Some(wanted) = &mut self.wanted {
            /* wanted may be ready now */
            self.try_advance_wanted_to_future(&mut events);
        }
        if let Some(train) = &mut self.future {
            /* future may have moved */
            train.set_carriages(&mut events);
        }
        if let Some(train) = &mut self.current {
            /* current may have moved */
            train.set_carriages(&mut events);
        }
        /* stuff may have happened above, tell dependents */
        self.dependents.carriages_loaded(self.quiescent_target(),&self.all_current_carriages(),&mut events);
        events
    }

    pub(super) fn update_dependents(&self) {
        self.dependents.busy(!(self.future.is_none() && self.wanted.is_none()));
    }

    pub(super) fn set_sketchy(&mut self, yn: bool) -> Result<RailwayEvents,DataMessage> {
        let mut events = RailwayEvents::new();
        self.sketchy = yn;
        if !yn {
            self.reset_position(&mut events)?;
        }
        Ok(events)
    }

    pub(super) fn invalidate(&mut self) -> Result<RailwayEvents,DataMessage> {
        let mut events = RailwayEvents::new();
        self.validity_counter += 1;
        /* create events */
        self.reset_position(&mut events)?;
        Ok(events)
    }
}
