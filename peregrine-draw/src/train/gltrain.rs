use peregrine_data::{ Carriage, CarriageId };
use std::collections::{ HashMap, HashSet };
use crate::{shape::layers::programstore::ProgramStore, util::needed::Needed};
use super::glcarriage::GLCarriage;
use crate::stage::stage::{ ReadStage };
use crate::webgl::DrawingSession;
use crate::webgl::global::WebGlGlobal;
use crate::shape::layers::drawingzmenus::ZMenuEvent;
use crate::util::message::Message;
#[cfg(blackbox)]
use blackbox::blackbox_log;

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

    pub(super) fn set_carriages(&mut self, new_carriages: &[Carriage], gl: &mut WebGlGlobal) -> Result<(),Message> {
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
                target.insert(carriage.id().clone(),GLCarriage::new(carriage,self.opacity,gl)?);
                redraw = true;
            }
        }
        self.carriages = target;
        if redraw {
            self.redraw_needed.set();
        }
        Ok(())
    }

    pub(crate) fn intersects(&self, stage: &ReadStage, mouse: (u32,u32)) -> Result<Option<ZMenuEvent>,Message> {
        for carriage in self.carriages.values() {
            if let Some(zmenu) = carriage.intersects(stage,mouse)? {
                return Ok(Some(zmenu));
            }
        }
        Ok(None)
    }

    pub(crate) fn intersects_fast(&self, stage: &ReadStage, mouse: (u32,u32)) -> Result<bool,Message> {
        for carriage in self.carriages.values() {
            if carriage.intersects_fast(stage,mouse)? {
                return Ok(true);
            }
        }
        Ok(false)
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
