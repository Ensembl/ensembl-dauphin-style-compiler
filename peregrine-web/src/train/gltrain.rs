use peregrine_core::{ Carriage, CarriageId };
use std::collections::{ HashMap, HashSet };
use crate::shape::layers::programstore::ProgramStore;
use super::glcarriage::GLCarriage;
use crate::shape::core::stage::{ ReadStage, RedrawNeeded };
use crate::webgl::DrawingSession;
use blackbox::blackbox_log;
use crate::webgl::global::WebGlGlobal;

pub struct GLTrain {
//    programs: ProgramStore,
    carriages: HashMap<CarriageId,GLCarriage>,
    opacity: f64,
    max: Option<u64>,
    redraw_needed: RedrawNeeded
}

impl GLTrain {
    pub fn new(programs: &ProgramStore, redraw_needed: &RedrawNeeded) -> GLTrain {
        GLTrain {
//            programs: programs.clone(),
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

    pub(super) fn discard(&mut self, gl: &mut WebGlGlobal) -> anyhow::Result<()> {
        blackbox_log!("gltrain","done(index={})",self.index);
        for (_,mut carriage) in self.carriages.drain() {
            carriage.discard(gl)?;
        }
        Ok(())
    }

    pub(super) fn set_carriages(&mut self, new_carriages: &[Carriage], gl: &mut WebGlGlobal) -> anyhow::Result<()> {
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

    pub fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, session: &DrawingSession) -> anyhow::Result<()> {
        for carriage in self.carriages.values_mut() {
            carriage.draw(gl,stage,session)?;
        }
        Ok(())
    }
}
