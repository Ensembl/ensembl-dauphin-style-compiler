use std::collections::HashMap;

use minicbor::{Encoder, Encode};

use crate::{command::{EarpOperand, EarpCommand}, error::EarpAssemblerError, setmapper::SetMapper, instructionset::EarpInstructionSetIdentifier, suite::Suite};

const EARPFILE_MAGIC : &str = "EARP0";

pub(crate) struct EarpFileWriter<'t> {
    set_mapper: SetMapper<'t>,
    entry_points: HashMap<String,i64>,
    instructions: Vec<EarpCommand>
}

impl<'t> EarpFileWriter<'t> {
    pub(crate) fn new(suite: &'t Suite) -> EarpFileWriter<'t> {
        EarpFileWriter {
            set_mapper: SetMapper::new(suite),
            entry_points: HashMap::new(),
            instructions: vec![]
        }
    }

    pub(crate) fn set_mapper_mut(&mut self) -> &mut SetMapper<'t> { &mut self.set_mapper }

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

impl<'t> Encode for EarpFileWriter<'t> {
    fn encode<W: minicbor::encode::Write>(&self, encoder: &mut Encoder<W>) -> Result<(), minicbor::encode::Error<W::Error>> {
        encoder.begin_map()?
            .str("M")?.str(EARPFILE_MAGIC)?
            .str("S")?.encode(&self.set_mapper)?
            .str("E")?.encode(&self.entry_points)?
            .str("I")?.encode(&self.instructions)?
            .end()?;
        Ok(())
    }
}
