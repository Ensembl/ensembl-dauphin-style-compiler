use peregrine_data::{ Scale, SingleHotspotEntry, SpecialClick };
use peregrine_toolkit::{lock};
use peregrine_toolkit_async::sync::needed::Needed;
use std::sync::{Arc, Mutex};
use super::glcarriage::GLCarriage;
use crate::stage::stage::{ ReadStage };
use crate::webgl::DrawingSession;
use crate::webgl::global::WebGlGlobal;
use crate::util::message::Message;

struct GLTrainData {
    carriages: Vec<GLCarriage>,
    opacity: f64,
    max: Option<u64>,
    redraw_needed: Needed
}

#[derive(Clone)]
pub struct GLTrain(Arc<Mutex<GLTrainData>>);

impl GLTrain {
    pub fn new(redraw_needed: &Needed) -> GLTrain {
        GLTrain(Arc::new(Mutex::new(GLTrainData {
            carriages: vec![],
            opacity: 0.,
            max: None,
            redraw_needed: redraw_needed.clone()
        })))
    }

    pub(super) fn scale(&self) -> Option<Scale> {
        lock!(self.0).carriages.iter().next().map(|c| c.extent().scale().clone())
    }

    pub(super) fn set_max(&mut self, max: u64) {
        lock!(self.0).max = Some(max);
    }

    pub(super) fn set_opacity(&mut self, amount: f64) {
        let mut state = lock!(self.0);
        state.redraw_needed.set();
        state.opacity = amount;
        for carriage in &state.carriages {
            carriage.set_opacity(amount);
        }
    }
    
    pub(super) fn set_carriages(&mut self, new_carriages: Vec<GLCarriage>) -> Result<(),Message> {
        let mut state = lock!(self.0);
        for c in &new_carriages {
            c.set_opacity(state.opacity);
        }
        state.carriages = new_carriages;
        state.redraw_needed.set();
        Ok(())
    }

    pub(crate) fn get_hotspot(&self, stage: &ReadStage, position: (f64,f64)) -> Result<Vec<SingleHotspotEntry>,Message> {
        let mut out = vec![];
        for carriage in &lock!(self.0).carriages {
            out.append(&mut carriage.get_hotspot(stage,position)?);
        }
        Ok(out)
    }

    pub(crate) fn special_hotspots(&self, stage: &ReadStage, position: (f64,f64)) -> Result<Vec<SpecialClick>,Message> {
        let mut out = vec![];
        for carriage in lock!(self.0).carriages.iter() {
            out.append(&mut carriage.special_hotspots(stage,position)?);
        }
        Ok(out)
    }

    pub(crate) fn draw(&mut self, gl: &Arc<Mutex<WebGlGlobal>>, stage: &ReadStage, session: &mut DrawingSession) -> Result<(),Message> {
        let mut carriages = lock!(self.0).carriages.iter().cloned().collect::<Vec<_>>();
        for mut carriage in carriages.drain(..) {
            let mut gl = lock!(gl);
            carriage.draw(&mut gl,stage,session)?;
        }
        Ok(())
    }
}
