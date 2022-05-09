use std::sync::{ Arc, Mutex, MutexGuard };
use identitynumber::{ identitynumber, hashable, orderable };
use lazy_static::lazy_static;
use crate::{ProgramName};
use crate::core::{ Layout, Scale };
use super::switch::SwitchOverlay;

identitynumber!(IDS);

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct Track {
    id: u64,
    min_scale: u64,
    max_scale: u64,
    scale_jump: u64,
    program_name: ProgramName,
    tags: Vec<String>,
    switch_overlay: Arc<Mutex<SwitchOverlay>>
}

hashable!(Track,id);
orderable!(Track,id);

impl Track {
    pub fn new(program_name: &ProgramName, min_scale: u64, max_scale: u64, scale_jump: u64) -> Track { 
        Track {
            id: IDS.next(),
            min_scale, max_scale, scale_jump,
            program_name: program_name.clone(),
            tags: vec![],
            switch_overlay: Arc::new(Mutex::new(SwitchOverlay::new()))
        }
    }

    pub fn add_tag(&mut self, tag: &str) { self.tags.push(tag.to_string()); }

    pub fn add_switch(&mut self, path: &[&str], yn: bool) {
        let mut switches = self.switch_overlay.lock().unwrap();
        if yn { switches.set(path); } else { switches.clear(path); }
    }

    pub(crate) fn overlay(&self) -> MutexGuard<SwitchOverlay> { self.switch_overlay.lock().unwrap() }
    pub fn program_name(&self) -> &ProgramName { &self.program_name }
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
