use identitynumber::identitynumber;
use lazy_static::lazy_static;
use crate::{ ProgramName };

identitynumber!(IDS);

#[derive(Clone,Debug,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub struct Track {
    id: u64,
    program_name: ProgramName
}

impl Track {
    pub fn new(program_name: &ProgramName) -> Track { 
        Track {
            id: IDS.next(),
            program_name: program_name.clone()
        }
    }

    pub fn program_name(&self) -> &ProgramName { &self.program_name }
    pub fn id(&self) -> u64 { self.id }
}
