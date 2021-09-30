use peregrine_data::{Assets, Carriage, CarriageId, Scale, ZMenuProxy};
use peregrine_toolkit::sync::needed::Needed;
use std::collections::{ HashMap, HashSet };
use std::rc::Rc;
use crate::{shape::layers::programstore::ProgramStore };
use super::glcarriage::GLCarriage;
use crate::stage::stage::{ ReadStage };
use crate::webgl::DrawingSession;
use crate::webgl::global::WebGlGlobal;
use crate::util::message::Message;

pub struct GLTrain {
    carriages: HashMap<CarriageId,GLCarriage>,
    opacity: f64,
    max: Option<u64>,
    redraw_needed: Needed
}

impl GLTrain {
    pub fn new(programs: &ProgramStore, redraw_needed: &Needed) -> GLTrain {
        GLTrain {
            carriages: HashMap::new(),
            opacity: 0.,
            max: None,
            redraw_needed: redraw_needed.clone()
        }
    }

    pub(super) fn set_max(&mut self, max: u64) {
        self.max = Some(max);
    }

    pub(super) fn set_opacity(&mut self, amount: f64) {
        self.redraw_needed.set();
        self.opacity = amount;
        for carriage in self.carriages.values() {
            carriage.set_opacity(amount);
        }
    }

    pub(super) fn discard(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        for (_,mut carriage) in self.carriages.drain() {
            carriage.discard(gl)?;
        }
        Ok(())
    }
    
    pub(super) fn set_carriages(&mut self, scale: &Scale, new_carriages: &[Carriage], gl: &mut WebGlGlobal, assets: &Assets) -> Result<(),Message> {
        let mut dont_keeps : HashSet<_> = self.carriages.keys().cloned().collect();
        let mut novels : HashSet<_> = new_carriages.iter().map(|x| x.id()).cloned().collect();
        for new in new_carriages {
            dont_keeps.remove(new.id());
        }
        for old in self.carriages.keys() {
            novels.remove(old);
        }
        let mut target = HashMap::new();
        for (id,mut carriage) in self.carriages.drain() {
            if dont_keeps.contains(&carriage.id()) {
                carriage.discard(gl)?;
            } else {
                target.insert(id,carriage);
            }
        }
        let mut redraw = false;
        for carriage in new_carriages {
            if novels.contains(carriage.id()) {
                target.insert(carriage.id().clone(),GLCarriage::new(carriage,scale,self.opacity,gl,assets)?);
                redraw = true;
            }
        }
        self.carriages = target;
        if redraw {
            self.redraw_needed.set();
        }
        Ok(())
    }

    pub(crate) fn get_hotspot(&self, stage: &ReadStage, position: (f64,f64)) -> Result<Vec<Rc<ZMenuProxy>>,Message> {
        let mut out = vec![];
        for (_,carriage) in self.carriages.iter() {
            out.append(&mut carriage.get_hotspot(stage,position)?);
        }
        Ok(out)
    }

    pub(crate) fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, session: &DrawingSession) -> Result<(),Message> {
        let mut min = 0;
        let mut max = 0;
        for carriage in self.carriages.values_mut() {
            let here_prio = carriage.priority_range();
            min = min.min(here_prio.0);
            max = max.max(here_prio.1);
        }
        for prio in min..(max+1) {
            for carriage in self.carriages.values_mut() {
                carriage.draw(gl,stage,session,prio)?;
            }
        }
        Ok(())
    }
}
