use peregrine_core::{ Carriage, CarriageId };
use std::collections::{ HashMap, HashSet };
use crate::shape::layers::programstore::ProgramStore;
use super::glcarriage::GLCarriage;
use crate::shape::core::stage::{ Stage, RedrawNeeded };
use blackbox::blackbox_log;

pub struct GLTrain {
    programs: ProgramStore,
    carriages: HashMap<CarriageId,GLCarriage>,
    opacity: f64,
    max: Option<u64>,
    redraw_needed: RedrawNeeded
}

impl GLTrain {
    pub fn new(programs: &ProgramStore, redraw_needed: &RedrawNeeded) -> GLTrain {
        GLTrain {
            programs: programs.clone(),
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

    pub(super) fn discard(&self) {
        blackbox_log!("gltrain","done(index={})",self.index);
    }

    pub(super) fn set_carriages(&mut self, new_carriages: &[Carriage]) -> anyhow::Result<()> {
        let mut dont_keeps : HashSet<_> = self.carriages.keys().cloned().collect();
        let mut novels : HashSet<_> = new_carriages.iter().map(|x| x.id()).cloned().collect();
        for new in new_carriages {
            dont_keeps.remove(new.id());
        }
        for old in self.carriages.keys() {
            novels.remove(old);
        }
        let mut target = HashMap::new();
        for (id,carriage) in self.carriages.drain() {
            if dont_keeps.contains(&carriage.id()) {
                carriage.destroy();
            } else {
                target.insert(id,carriage);
            }
        }
        let mut redraw = false;
        for carriage in new_carriages {
            if novels.contains(carriage.id()) {
                target.insert(carriage.id().clone(),GLCarriage::new(carriage,self.opacity,&self.programs)?);
                redraw = true;
            }
        }
        self.carriages = target;
        if redraw {
            self.redraw_needed.set();
        }
        Ok(())
    }

    pub fn draw(&mut self, stage: &Stage) -> anyhow::Result<()> {
        for carriage in self.carriages.values_mut() {
            carriage.draw(stage)?;
        }
        Ok(())
    }
}
