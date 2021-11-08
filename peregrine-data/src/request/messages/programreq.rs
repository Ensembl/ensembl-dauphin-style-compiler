use crate::{ProgramName, request::core::request::{RequestVariant}};
use serde_cbor::Value as CborValue;

pub(crate) struct ProgramReq {
    program_name: ProgramName
}

impl ProgramReq {
    pub(crate) fn new(program_name: &ProgramName) -> RequestVariant {
        RequestVariant::Program(ProgramReq {
            program_name: program_name.clone()
        })
    }
    
    pub(crate) fn encode(&self) -> CborValue {
        self.program_name.encode()
    }
}
