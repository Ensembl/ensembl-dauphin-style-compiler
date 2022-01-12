use std::{collections::HashMap};

use crate::{instructionset::{InstructionSetId}, error::{AssemblerError, unknown_opcode_error, duplicate_opcode_error}, setmapper::SetMapper};

pub(crate) struct Lookup {
    mappings: HashMap<Option<String>,Vec<InstructionSetId>>,
    cache: HashMap<(Option<String>,String),Option<u64>>
}

impl Lookup {
    pub(crate) fn new() -> Lookup {
        Lookup {
            mappings: HashMap::new(),
            cache: HashMap::new()
        }
    }

    pub(crate) fn add_mapping(&mut self, prefix: &Option<String>, id: &InstructionSetId) {
        self.mappings.entry(prefix.clone()).or_insert(vec![]).push(id.clone());
        self.cache.clear();
    }

    fn real_lookup(&mut self, set_mapper: &mut SetMapper, prefix: &Option<String>, name: &str) -> Result<Option<u64>,AssemblerError> {
        let empty = vec![];
        let identifiers = self.mappings.get(&prefix).unwrap_or(&empty);
        let mut out = None;
        for id in identifiers {
            if let Some(opcode) = set_mapper.lookup(id,name) {
                if out.is_some() {
                    return Err(duplicate_opcode_error(prefix,name));
                }
                out = Some(opcode);
            }
        }
        Ok(out)
    }

    fn cached_lookup(&mut self, set_mapper: &mut SetMapper, prefix: &Option<String>, name: &str) -> Result<Option<u64>,AssemblerError> {
        let key = (prefix.clone(),name.to_string());
        if let Some(opcode) = self.cache.get(&key) {
            return Ok(opcode.clone());
        }
        let opcode = self.real_lookup(set_mapper,&prefix,name)?;
        self.cache.insert(key,opcode);
        Ok(opcode)
    }

    pub(crate) fn lookup(&mut self, set_mapper: &mut SetMapper, prefix: &Option<&str>, name: &str) -> Result<u64,AssemblerError> {
        let prefix = prefix.map(|x| x.to_string());
        self.cached_lookup(set_mapper,&prefix,name)?.ok_or_else(|| unknown_opcode_error(&prefix,name))
    }
}
