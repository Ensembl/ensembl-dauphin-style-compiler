use crate::{request::core::minirequest::{MiniRequest, MiniRequestVariety}, shapeload::programname::ProgramName};
use serde::Serialize;

pub struct ProgramReq {
    program_name: ProgramName
}

impl ProgramReq {
    pub(crate) fn new(program_name: &ProgramName) -> MiniRequest {
        MiniRequest::Program(ProgramReq {
            program_name: program_name.clone()
        })
    }

    pub fn name(&self) -> &ProgramName { &self.program_name }
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
