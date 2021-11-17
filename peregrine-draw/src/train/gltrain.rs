use peregrine_data::{Assets, Carriage, CarriageExtent, Scale, ZMenuProxy};
use peregrine_toolkit::sync::needed::Needed;
use std::collections::{ HashMap, HashSet };
use std::rc::Rc;
use super::glcarriage::GLCarriage;
use crate::stage::stage::{ ReadStage };
use crate::webgl::DrawingSession;
use crate::webgl::global::WebGlGlobal;
use crate::util::message::Message;

pub struct GLTrain {
    carriages: HashMap<CarriageExtent,GLCarriage>,
    opacity: f64,
    max: Option<u64>,
    redraw_needed: Needed
}

impl GLTrain {
    pub fn new(redraw_needed: &Needed) -> GLTrain {
        GLTrain {
            carriages: HashMap::new(),
            opacity: 0.,
            max: None,
            redraw_needed: redraw_needed.clone()
        }
    }

    pub(super) fn scale(&self) -> Option<Scale> {
        self.carriages.iter().next().map(|(x,_)| x.train().scale()).cloned()
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
    
    pub(super) fn set_carriages(&mut self, new_carriages: &[Carriage], gl: &mut WebGlGlobal, assets: &Assets) -> Result<(),Message> {
        let mut dont_keeps : HashSet<_> = self.carriages.keys().cloned().collect();
        let mut novels : HashSet<_> = new_carriages.iter().map(|x| x.extent()).cloned().collect();
        for new in new_carriages {
            dont_keeps.remove(new.extent());
        }
        for old in self.carriages.keys() {
            novels.remove(old);
        }
        let mut target = HashMap::new();
        for (id,mut carriage) in self.carriages.drain() {
            if dont_keeps.contains(&carriage.extent()) {
                carriage.discard(gl)?;
            } else {
                target.insert(id,carriage);
            }
        }
        let mut redraw = false;
        for carriage in new_carriages {
            if novels.contains(carriage.extent()) {
                target.insert(carriage.extent().clone(),GLCarriage::new(carriage,self.opacity,gl,assets)?);
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

    pub(crate) fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, session: &mut DrawingSession) -> Result<(),Message> {
        for carriage in self.carriages.values_mut() {
            carriage.draw(gl,stage,session)?;
        }
        Ok(())
    }
}
