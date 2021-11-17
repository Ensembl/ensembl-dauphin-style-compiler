use peregrine_toolkit::sync::blocker::{Blocker};
use crate::{CarriageSpeed, LaneStore, PeregrineCoreBase};
use crate::api::MessageSender;
use crate::core::{ Scale, Viewport };
use super::railwaydependents::RailwayDependents;
use super::train::{ Train };
use super::carriage::{Carriage};
use super::railwayevent::RailwayEvents;
use super::trainextent::TrainExtent;
use crate::util::message::DataMessage;

/* current: train currently being displayed, if any. During transition, the outgoing train.
 * future: incoming train during transition.
 * wanted: train to be displayed when complete and when transitions are done.
 */

pub struct TrainSet {
    current: Option<Train>,
    future: Option<Train>,
    wanted: Option<Train>,
    next_activation: u32,
    messages: MessageSender,
    dependents: RailwayDependents,
}

impl TrainSet {
    pub(super) fn new(base: &PeregrineCoreBase, result_store: &LaneStore, visual_blocker: &Blocker) -> TrainSet {
        TrainSet {
            current: None,
            future: None,
            wanted: None,
            next_activation: 0,
            messages: base.messages.clone(),
            dependents: RailwayDependents::new(base,result_store,visual_blocker),
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
        if self.wanted.as_ref().map(|x| x.train_ready() && !x.train_broken()).unwrap_or(false) && self.future.is_none() {
            if let Some(mut wanted) = self.wanted.take() {
                let speed = self.current.as_ref().map(|x| x.extent().speed_limit(&wanted.extent())).unwrap_or(CarriageSpeed::Quick);
                wanted.set_active(events,self.next_activation,speed);
                self.next_activation += 1;
                self.future = Some(wanted);
                self.dependents.carriages_loaded(self.quiescent_target(),&self.all_current_carriages(),events);
                self.draw_notify_viewport(events);
            }
        }
    }

    fn draw_notify_viewport(&self, events: &mut RailwayEvents) {
        if let Some(train) = self.future.as_ref().or_else(|| self.current.as_ref()) {
            events.draw_notify_viewport(&train.viewport(),false);
        }
    }

    fn maybe_new_wanted(&mut self, events: &mut RailwayEvents, viewport: &Viewport) -> Result<(),DataMessage> {
        let train_id = TrainExtent::new(viewport.layout()?,&Scale::new_bp_per_screen(viewport.bp_per_screen()?));
        let mut new_target_needed = true;
        if let Some(quiescent) = self.quiescent_target() {
            if quiescent.extent() == train_id {
                new_target_needed = false;
            }
        }
        if new_target_needed {
            self.wanted = Some(Train::new(&train_id,events,viewport,&self.messages)?);
        }
        Ok(())
    }

    pub(super) fn set_position(&mut self, events: &mut RailwayEvents, viewport: &Viewport) -> Result<(),DataMessage> {
        if !viewport.ready() { return Ok(()); }
        /* maybe we need to change where we're heading? */
        self.maybe_new_wanted(events,viewport)?;
        /* dependents need to know we moved */
        if let Some(train) = self.quiescent_target() {
            self.dependents.position_was_updated(train,viewport.position()?);
        }
        /* All the trains get the new position */
        let viewport_stick = viewport.layout()?.stick();
        self.each_current_train(events,&|events,train| {
            if viewport_stick == train.extent().layout().stick() {
                train.set_position(&mut events.clone(),viewport); // XXX error handling
            }
        });
        /* check if any progress can be made */
        self.try_advance_wanted_to_future(events);
        /* tell dependents */
        self.draw_notify_viewport(events);
        Ok(())
    }

    pub(super) fn transition_complete(&mut self, events: &mut RailwayEvents) {
        /* retire current and make future current */
        if let Some(mut current) = self.current.take() {
            current.set_inactive();
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
            wanted.check_if_ready();
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
}
