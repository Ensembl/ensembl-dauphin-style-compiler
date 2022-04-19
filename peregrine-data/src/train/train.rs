use std::sync::{ Arc, Mutex };
use peregrine_toolkit::puzzle::AnswerAllocator;
use peregrine_toolkit::{lock, log};
use peregrine_toolkit::sync::needed::Needed;
use crate::api::{CarriageSpeed, MessageSender };
use super::drawingcarriage::DrawingCarriage2;
use super::graphics::Graphics;
use super::railwaydatatasks::RailwayDataTasks;
use super::carriageset::{CarriageSet};
use super::trainextent::TrainExtent;
use crate::util::message::DataMessage;
use crate::switch::trackconfiglist::TrainTrackConfigList;
use crate::core::Viewport;

pub(super) enum StickData {
    Pending,
    Ready(u64),
    Unavailable
}

impl StickData {
    fn is_broken(&self) -> bool { match self { StickData::Unavailable => true, _ => false } }
    fn is_ready(&self) -> bool { match self { StickData::Ready(_) => true, _ => false } }
}

// XXX circular chroms
pub(super) struct Train {
    extent: TrainExtent,
    max: Arc<Mutex<StickData>>,
    viewport: Option<Viewport>,
    carriages: CarriageSet,
    graphics: Graphics,
}

impl Train {
    pub(super) fn new(graphics: &Graphics, ping_needed: &Needed, answer_allocator: &Arc<Mutex<AnswerAllocator>>, extent: &TrainExtent, carriage_loader: &RailwayDataTasks, messages: &MessageSender) -> Result<Train,DataMessage> {
        let train_track_config_list = TrainTrackConfigList::new(&extent.layout(),&extent.scale());
        let out = Train {
            max: Arc::new(Mutex::new(StickData::Pending)),
            extent: extent.clone(),
            graphics: graphics.clone(),
            viewport: None,
            carriages: CarriageSet::new(&ping_needed, answer_allocator,extent,&train_track_config_list,carriage_loader,graphics,messages),
        };
        carriage_loader.add_stick(&out.extent(),&out.stick_data_holder());
        Ok(out)
    }

    pub(super) fn ping(&mut self) {
        self.carriages.ping();
    }

    pub(super) fn each_current_drawing_carriage<X,F>(&self, state: &mut X, cb: &F) where F: Fn(&mut X,&DrawingCarriage2) {
        self.carriages.each_current_drawing_carriage(state,cb);
    }

    pub(super) fn speed_limit(&self, other: &Train) -> CarriageSpeed {
        if self.extent() == other.extent() {
            self.extent().speed_limit(&other.extent())
        } else {
            CarriageSpeed::Slow
        }
    }

    pub(super) fn extent(&self) -> &TrainExtent { &self.extent }
    pub(super) fn viewport(&self) -> Option<&Viewport> { self.viewport.as_ref() }
    pub(super) fn is_active(&self) -> bool { self.carriages.is_active() }
    pub(super) fn train_ready(&self) -> bool { 
        self.train_half_ready() && self.carriages.all_ready() 
    }

    pub(super) fn train_half_ready(&self) -> bool {
        self.carriages.central_drawing_carriage().is_some() && lock!(self.max).is_ready()
    }

    pub(super) fn train_broken(&self) -> bool { lock!(self.max).is_broken() }

    pub(super) fn set_active(&mut self, speed: CarriageSpeed) {
        let max = match &*lock!(self.max) {
            StickData::Ready(max) => *max,
            _ => { panic!("set_active() called on non-ready train") }
        };
        self.carriages.set_active(true);
        self.graphics.start_transition(&self.extent,max,speed);
    }

    pub(super) fn set_inactive(&mut self) {
        self.carriages.set_active(false);
    }

    pub(super) fn set_position(&mut self, viewport: &Viewport) -> Result<(),DataMessage> {
        log!("set poisition {:?}",viewport);
        let centre_carriage_index = self.extent.scale().carriage(viewport.position()?);
        self.carriages.update_centre(centre_carriage_index);
        self.viewport = Some(viewport.clone());
        Ok(())
    }
    
    pub(super) fn stick_data_holder(&self) -> &Arc<Mutex<StickData>> { &self.max }
}
