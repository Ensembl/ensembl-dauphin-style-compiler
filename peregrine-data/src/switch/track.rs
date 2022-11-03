use std::sync::{ Arc, Mutex, MutexGuard };
use identitynumber::{ identitynumber, hashable, orderable };
use lazy_static::lazy_static;
use peregrine_toolkit::eachorevery::eoestruct::StructBuilt;
use peregrine_toolkit::lock;
use crate::core::{ Layout, Scale };
use crate::shapeload::programname::ProgramName;
use super::switchoverlay::SwitchOverlay;

identitynumber!(IDS);

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct Track {
    id: u64,
    min_scale: u64,
    max_scale: u64,
    scale_jump: u64,
    program: ProgramName,
    tags: Vec<String>,
    switch_overlay: Arc<Mutex<SwitchOverlay>>
}

hashable!(Track,id);
orderable!(Track,id);

impl Track {
    pub fn new(program: &ProgramName, min_scale: u64, max_scale: u64, scale_jump: u64) -> Track { 
        Track {
            id: IDS.next(),
            min_scale, max_scale, scale_jump,
            program: program.clone(),
            tags: vec![],
            switch_overlay: Arc::new(Mutex::new(SwitchOverlay::new()))
        }
    }

    pub fn add_tag(&mut self, tag: &str) { self.tags.push(tag.to_string()); }

    pub fn set_switch(&mut self, path: &[&str], value: StructBuilt) {
        let mut switches = lock!(self.switch_overlay);
        if !value.is_null() { switches.set(path,value); } else { switches.clear(path); } // XXX single call
    }

    pub(super) fn overlay(&self) -> MutexGuard<SwitchOverlay> { self.switch_overlay.lock().unwrap() }

    pub fn program(&self) -> &ProgramName { &self.program }
    pub fn id(&self) -> u64 { self.id }
    pub fn scale(&self) -> (u64,u64) { (self.min_scale,self.max_scale) }
    pub fn max_scale_jump(&self) -> u64 { self.scale_jump }
    pub fn tags(&self) -> &[String] { &self.tags }

    pub fn best_scale(&self, request: &Scale) -> Option<Scale> {
        let request = request.get_index();
        if request < self.min_scale || request >= self.max_scale { return None; }
        let end = self.max_scale-1;
        Some(Scale::new(end-((end-request)/self.scale_jump)*self.scale_jump))
    }

    pub fn available(&self, _layout: &Layout, scale: &Scale) -> bool {
        // XXX filter on layout
        let want_scale =scale.get_index();
        if want_scale < self.min_scale || want_scale >= self.max_scale { return false; }
        true
    }
}
