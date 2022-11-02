use crate::{request::core::request::{MiniRequest, MiniRequestVariety}, shapeload::programname::ProgramName};
use peregrine_toolkit::log;
use serde::Serialize;

pub struct ProgramReq {
    program_name: ProgramName
}

impl ProgramReq {
    pub(crate) fn new(program_name: &ProgramName) -> MiniRequest {
        log!("ProgramReq({:?})",program_name);
        MiniRequest::Program(ProgramReq {
            program_name: program_name.clone()
        })
    }    
}

impl MiniRequestVariety for ProgramReq {
    fn description(&self) -> String { "program".to_string() }
    fn opcode(&self) -> u8 { 1 }
}

impl Serialize for ProgramReq {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        self.program_name.serialize(serializer)
    }
}