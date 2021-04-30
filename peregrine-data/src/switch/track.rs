use identitynumber::identitynumber;
use lazy_static::lazy_static;
use crate::{ ProgramName };
use crate::core::{ Layout, Scale };

identitynumber!(IDS);

#[derive(Clone,Debug,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub struct Track {
    id: u64,
    min_scale: u64,
    max_scale: u64,
    scale_jump: u64,
    program_name: ProgramName,
    tags: Vec<String>
}

impl Track {
    pub fn new(program_name: &ProgramName, min_scale: u64, max_scale: u64, scale_jump: u64) -> Track { 
        Track {
            id: IDS.next(),
            min_scale, max_scale, scale_jump,
            program_name: program_name.clone(),
            tags: vec![]
        }
    }

    pub fn add_tag(&mut self, tag: &str) { self.tags.push(tag.to_string()); }
    pub fn program_name(&self) -> &ProgramName { &self.program_name }
    pub fn id(&self) -> u64 { self.id }
    pub fn scale(&self) -> (u64,u64) { (self.min_scale,self.max_scale) }
    pub fn max_scale_jump(&self) -> u64 { self.scale_jump }
    pub fn tags(&self) -> &[String] { &self.tags }

    pub fn best_scale(&self, request: Scale) -> Option<Scale> {
        let request = request.get_index();
        if request < self.min_scale || request >= self.max_scale { return None; }
        let end = self.max_scale-1;
        Some(Scale::new((end-(end-request)/self.scale_jump)*self.scale_jump))
    }

    pub fn available(&self, layout: &Layout, scale: &Scale) -> bool {
        let want_scale =scale.get_index();
        if want_scale < self.min_scale || want_scale >= self.max_scale { return false; }
        true
    }
}
