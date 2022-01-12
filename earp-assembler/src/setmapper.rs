use minicbor::{Encode, Encoder};
use std::collections::HashMap;

use crate::{instructionset::{InstructionSetId, InstructionSet}, suite::Suite};

pub(crate) struct SetMapper<'t> {
    offsets: HashMap<InstructionSetId,u64>,
    next_offset: u64,
    suite: &'t Suite
}

impl<'t> SetMapper<'t> {
    pub(crate) fn new(suite: &'t Suite) -> SetMapper<'t> {
        SetMapper {
            offsets: HashMap::new(),
            next_offset: 0,
            suite
        }
    }

    fn offset_for(&mut self, set: &InstructionSet) -> u64 {
        if !self.offsets.contains_key(set.identifier()) {
            self.offsets.insert(set.identifier().clone(),self.next_offset);
            self.next_offset += set.next_opcode();    
        }
        *self.offsets.get(set.identifier()).unwrap()
    }

    fn lookup_real(&mut self, set: &InstructionSet, name: &str) -> Option<u64> {
        set.lookup(name).map(|x| x+self.offset_for(set))
    }

    pub(crate) fn lookup(&mut self, id: &InstructionSetId, name: &str) -> Option<u64> {
        if let Some(set) = self.suite.get_instruction_set(id) {
            self.lookup_real(set,name)
        } else {
            None
        }
    }
}

impl<'t> Encode for SetMapper<'t> {
    fn encode<W: minicbor::encode::Write>(&self, encoder: &mut Encoder<W>) -> Result<(), minicbor::encode::Error<W::Error>> {
        encoder.begin_array()?;
        let mut ids = self.offsets.keys().collect::<Vec<_>>();
        ids.sort();
        for id in &ids {
            let offset = self.offsets.get(id).unwrap();
            encoder.str(&id.0)?;
            encoder.u64(id.1)?;
            encoder.u64(*offset)?;
        }
        encoder.end()?;
        Ok(())
    }
}
