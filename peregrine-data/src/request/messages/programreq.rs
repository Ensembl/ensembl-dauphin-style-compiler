use serde::Serializer;

use crate::{ProgramName, request::core::request::NewRequestVariant};

pub(crate) struct ProgramCommandRequest {
    program_name: ProgramName
}

impl ProgramCommandRequest {
    pub(crate) fn new(program_name: &ProgramName) -> NewRequestVariant {
        NewRequestVariant::Program(ProgramCommandRequest {
            program_name: program_name.clone()
        })
    }
}

impl serde::Serialize for ProgramCommandRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        self.program_name.serialize(serializer)
    }
}
