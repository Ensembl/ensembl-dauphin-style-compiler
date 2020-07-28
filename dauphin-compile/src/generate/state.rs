use std::collections::HashMap;
use dauphin_interp::runtime::{ Register };
use dauphin_interp::util::DauphinError;
use crate::model::{ DefStore, RegisterAllocator };
use crate::typeinf::{ TypeModel, Typing };

#[derive(Debug)]
pub struct CodeGenRegNames {
    output: HashMap<String,Register>,
    input: HashMap<String,Register>
}

impl CodeGenRegNames {
    pub fn new() -> CodeGenRegNames {
        CodeGenRegNames {
            input: HashMap::new(),
            output: HashMap::new()
        }
    }

    pub fn lookup_input(&self, id: &str) -> anyhow::Result<&Register> {
        if !self.input.contains_key(id) {
            return Err(DauphinError::source(&format!("No such variable {:?}",id)));
        }
        Ok(&self.input[id])
    }

    pub fn lookup_output(&mut self, id: &str, stomp: bool, alloc: &RegisterAllocator) -> anyhow::Result<Register> {
        if stomp {
            // if it's a top level assignment allow type change
            self.output.remove(id);
        }
        if !self.output.contains_key(id) {
            self.output.insert(id.to_string(),alloc.allocate());
        }
        Ok(self.output[id])
    }

    pub fn commit(&mut self) {
        self.input.extend(self.output.drain());
        self.output = self.input.clone();
    }
}

pub struct GenerateState<'a> {
    codegen_regnames: CodeGenRegNames,
    types: TypeModel,
    typing: Typing,
    regalloc: RegisterAllocator,
    defstore: &'a DefStore
}

impl<'a> GenerateState<'a> {
    pub fn new(defstore: &'a DefStore) -> GenerateState<'a> {
        GenerateState {
            codegen_regnames: CodeGenRegNames::new(),
            types: TypeModel::new(),
            typing: Typing::new(),
            defstore,
            regalloc: RegisterAllocator::new(0)
        }
    }

    pub fn regalloc(&self) -> &RegisterAllocator { &self.regalloc }
    pub fn types(&mut self) -> &mut TypeModel { &mut self.types }
    pub fn typing(&mut self) -> &mut Typing { &mut self.typing }
    pub fn defstore(&self) -> &'a DefStore { self.defstore }
    pub fn codegen_regnames(&mut self) -> &mut CodeGenRegNames { &mut self.codegen_regnames }
}
