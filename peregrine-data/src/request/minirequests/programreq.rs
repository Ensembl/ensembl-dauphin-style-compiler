use crate::{ProgramName, request::core::request::{MiniRequest, MiniRequestVariety}};
use serde_cbor::Value as CborValue;

pub struct ProgramReq {
    program_name: ProgramName
}

impl ProgramReq {
    pub(crate) fn new(program_name: &ProgramName) -> MiniRequest {
        MiniRequest::Program(ProgramReq {
            program_name: program_name.clone()
        })
    }
    
    pub fn encode(&self) -> CborValue {
        self.program_name.encode()
    }
}

impl MiniRequestVariety for ProgramReq {
    fn description(&self) -> String { "program".to_string() }
}
