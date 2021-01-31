use peregrine_core::Carriage;
use std::collections::HashSet;
use crate::shape::layers::programstore::ProgramStore;
use super::glcarriage::GLCarriage;
use blackbox::blackbox_log;

pub struct GLTrain {
    programs: ProgramStore,
    carriages: HashSet<GLCarriage>,
    index: u32,
    opacity: f64,
    max: Option<u64>
}

impl GLTrain {
    pub fn new(index: u32, programs: &ProgramStore) -> GLTrain {
        GLTrain {
            programs: programs.clone(),
            carriages: HashSet::new(),
            index,
            opacity: 0.,
            max: None
        }
    }

    pub fn index(&self) -> u32 { self.index }

    pub(super) fn set_max(&mut self, max: u64) {
        self.max = Some(max);
    }

    pub(super) fn set_opacity(&mut self, amount: f64) {
        self.opacity = amount;
        for carriage in &mut self.carriages.iter() {
            carriage.set_opacity(amount);
        }
    }

    pub(super) fn discard(&self) {
        blackbox_log!("gltrain","done(index={})",self.index);
    }

    pub(super) fn set_carriages(&mut self, new_carriages: &[Carriage]) -> anyhow::Result<()> {
        let carriages : Vec<_> = new_carriages.iter().map(|x| x.id().to_string()).collect();
        blackbox_log!("gltrain","set_carriges(carriages={:?} index={})",carriages,self.index);
        let mut keeps : HashSet<_> = self.carriages.iter().map(|x| x.id()).cloned().collect();
        let mut novels : HashSet<_> = new_carriages.iter().map(|x| x.id()).cloned().collect();
        for new in new_carriages {
            keeps.remove(new.id());
        }
        for old in &self.carriages {
            novels.remove(old.id());
        }
        let mut target = vec![];
        for carriage in self.carriages.drain() {
            if keeps.contains(&carriage.id()) {
                target.push(carriage);
            } else {
                carriage.destroy();
            }
        }
        for carriage in new_carriages {
            if novels.contains(carriage.id()) {
                target.push(GLCarriage::new(carriage,self.opacity,&self.programs)?);
            }
        }
        self.carriages = target.drain(..).collect();
        Ok(())
    }
}
