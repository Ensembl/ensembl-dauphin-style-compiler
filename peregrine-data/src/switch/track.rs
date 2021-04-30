use identitynumber::identitynumber;
use lazy_static::lazy_static;
use crate::{ ProgramName };

identitynumber!(IDS);

#[derive(Clone,Debug,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub struct Track {
    id: u64,
    min_scale: u64,
    max_scale: u64,
    scale_jump: u64,
    program_name: ProgramName
}

impl Track {
    pub fn new(program_name: &ProgramName, min_scale: u64, max_scale: u64, scale_jump: u64) -> Track { 
        Track {
            id: IDS.next(),
            min_scale, max_scale, scale_jump,
            program_name: program_name.clone()
        }
    }

    pub fn program_name(&self) -> &ProgramName { &self.program_name }
    pub fn id(&self) -> u64 { self.id }
}
