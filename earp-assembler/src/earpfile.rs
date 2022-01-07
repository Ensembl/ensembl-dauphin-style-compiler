use std::collections::HashMap;

use minicbor::{Encoder, Encode};

use crate::{command::{EarpOperand, EarpCommand}, error::EarpAssemblerError};

const EARPFILE_MAGIC : &str = "EARP0";

pub struct EarpFileWriter {
    entry_points: HashMap<String,i64>,
    instructions: Vec<EarpCommand>
}

impl EarpFileWriter {
    pub fn new() -> EarpFileWriter {
        EarpFileWriter {
            entry_points: HashMap::new(),
            instructions: vec![]
        }
    }

    pub(crate) fn add_instruction(&mut self, opcode: u64, operands: &[EarpOperand]) {
        self.instructions.push(EarpCommand(opcode,operands.iter().cloned().collect()));
    }

    pub(crate) fn add_entry_point(&mut self, name: &str, pc: i64) {
        self.entry_points.insert(name.to_string(),pc);
    }

    pub(crate) fn assemble(&self) -> Result<Vec<u8>,EarpAssemblerError> {
        let mut bytes = vec![];
        let mut encoder = Encoder::new(&mut bytes);
        encoder.encode(self).map_err(|e| EarpAssemblerError::EncodingError(e.to_string()))?;
        Ok(bytes)
    }
}

impl Encode for EarpFileWriter {
    fn encode<W: minicbor::encode::Write>(&self, encoder: &mut Encoder<W>) -> Result<(), minicbor::encode::Error<W::Error>> {
        encoder.begin_map()?
            .str("M")?.str(EARPFILE_MAGIC)?
            .str("E")?.encode(&self.entry_points)?
            .str("I")?.encode(&self.instructions)?
            .end()?;
        Ok(())
    }
}
