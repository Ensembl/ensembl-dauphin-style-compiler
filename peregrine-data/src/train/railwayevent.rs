/*
use std::sync::{ Arc, Mutex };
use peregrine_toolkit::lock;
use peregrine_toolkit::sync::needed::Needed;

use crate::{PlayingField, TrainExtent, PeregrineCoreBase};
use crate::api::{ CarriageSpeed, ApiMessage };
use crate::core::Viewport;

enum RailwayEvent {
}

#[derive(Clone)]
pub(crate) struct RailwayEvents(Arc<Mutex<Vec<RailwayEvent>>>,Needed);

impl RailwayEvents {
    pub(super) fn new(try_lifecycle: &Needed) -> RailwayEvents {
        RailwayEvents(Arc::new(Mutex::new(vec![])),try_lifecycle.clone())
    }

    pub fn lifecycle(&self) -> &Needed { &self.1 }

    #[allow(unused)]
    pub fn len(&self) -> usize { lock!(self.0).len() }

    pub(super) fn run_events(&mut self, base: &mut PeregrineCoreBase) {
        let events : Vec<RailwayEvent> = self.0.lock().unwrap().drain(..).collect();
        let mut errors = vec![];
        let mut transition = None; /* delay till after corresponding set also eat multiples */
        let mut notifications = vec![];
        for e in events {
            match e {
                RailwayEvent::DrawNotifyViewport(viewport, future) => {
                    notifications.push((viewport,future));
                },
            }
        }
        if let Some((train,max,speed)) = transition {
            let r = base.integration.lock().unwrap().start_transition(&train,max,speed);
            if let Err(r) = r {
                errors.push(r);
                base.queue.push(ApiMessage::TransitionComplete);
            }
        }
        let mut integration =  base.integration.lock().unwrap();
        for (viewport,future) in notifications.drain(..) {
            if !future {
                integration.notify_viewport(&viewport);
            }
        }
        for error in errors.drain(..) {
            base.messages.send(error);
        }
    }
}
*/
