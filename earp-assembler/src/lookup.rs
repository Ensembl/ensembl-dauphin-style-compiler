use std::{collections::HashMap};

use crate::{instructionset::{EarpInstructionSetIdentifier}, error::{EarpAssemblerError, opcode_error}, suite::Suite, setmapper::SetMapper};

pub(crate) struct Lookup {
    mappings: HashMap<Option<String>,Vec<EarpInstructionSetIdentifier>>,
    cache: HashMap<(Option<String>,String),Option<u64>>
}

impl Lookup {
    pub(crate) fn new() -> Lookup {
        Lookup {
            mappings: HashMap::new(),
            cache: HashMap::new()
        }
    }

    pub(crate) fn add_mapping(&mut self, prefix: &Option<String>, id: &EarpInstructionSetIdentifier) {
        self.mappings.entry(prefix.clone()).or_insert(vec![]).push(id.clone());
        self.cache.clear();
    }

    fn real_lookup(&mut self, set_mapper: &mut SetMapper, prefix: &Option<String>, name: &str) -> Option<u64> {
        let empty = vec![];
        let identifiers = self.mappings.get(&prefix).unwrap_or(&empty);
        for id in identifiers {
            if let Some(opcode) = set_mapper.lookup(id,name) {
                return Some(opcode)
            }
        }
        None
    }

    fn cached_lookup(&mut self, set_mapper: &mut SetMapper, prefix: &Option<String>, name: &str) -> Option<u64> {
        let key = (prefix.clone(),name.to_string());
        if let Some(opcode) = self.cache.get(&key) {
            return opcode.clone();
        }
        let opcode = self.real_lookup(set_mapper,&prefix,name);
        self.cache.insert(key,opcode);
        opcode
    }

    pub(crate) fn lookup(&mut self, set_mapper: &mut SetMapper, prefix: &Option<&str>, name: &str) -> Result<u64,EarpAssemblerError> {
        let prefix = prefix.map(|x| x.to_string());
        self.cached_lookup(set_mapper,&prefix,name).ok_or_else(|| opcode_error(prefix,name))
    }
}
