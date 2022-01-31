use std::{collections::{HashMap, BTreeMap}, sync::Mutex};

use crate::{suite::{suite::Suite, instructionset::InstructionSetId}, runtime::command::Command, core::error::EarpRuntimeError};

pub struct Resolver<'a> {
    suite: &'a Suite,
    sets: BTreeMap<u64,InstructionSetId>,
    cache: Mutex<HashMap<u64,Option<Command>>>
}

impl<'a> Resolver<'a> {
    fn make_sets(input: &[(String,u64,u64)]) -> BTreeMap<u64,InstructionSetId> {
        let mut output = BTreeMap::new();
        for (name,version,offset) in input {
            output.insert(*offset,InstructionSetId(name.to_string(),*version));
        }
        output
    }

    pub fn new(suite: &'a Suite, sets: &[(String,u64,u64)]) -> Resolver<'a> {
        Resolver {
            suite,
            sets: Resolver::make_sets(sets),
            cache: Mutex::new(HashMap::new())
        }
    }

    pub fn real_lookup(&self, offset: u64) -> Option<Command> {
        self.sets.range(..(offset+1)).rev().next().and_then(|(base,id)| {
            self.suite.lookup(id,offset-base)
        }).cloned()
    }
    
    pub fn cached_lookup(&self, offset: u64) -> Option<Command> {
        self.cache.lock().unwrap().entry(offset).or_insert_with(||
            self.real_lookup(offset)
        ).clone()
    }

    pub fn lookup(&self, offset: u64) -> Result<Command,EarpRuntimeError> {
        self.cached_lookup(offset).ok_or_else(||
            EarpRuntimeError::BadOpcode(format!("offset"))
        )
    }
}