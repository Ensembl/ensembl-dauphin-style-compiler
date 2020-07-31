use std::collections::HashMap;
use dauphin_interp::runtime::{ Register };
use dauphin_interp::util::DauphinError;
use crate::model::{ DefStore, RegisterAllocator };
use crate::typeinf::{ TypeModel, Typing };
use crate::generate::simplify::SimplifyMapperData;
use crate::generate::linearize::LinearizeRegsData;

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

pub struct GenerateState {
    debug_name: String,
    codegen_regnames: CodeGenRegNames,
    types: TypeModel,
    typing: Typing,
    regalloc: RegisterAllocator,
    defstore: DefStore,
    simplify_mapper: SimplifyMapperData,
    linearize_regs: LinearizeRegsData
}

impl GenerateState {
    pub fn new(debug_name: &str) -> GenerateState {
        GenerateState {
            codegen_regnames: CodeGenRegNames::new(),
            debug_name: debug_name.to_string(),
            types: TypeModel::new(),
            typing: Typing::new(),
            regalloc: RegisterAllocator::new(0),
            defstore: DefStore::new(),
            simplify_mapper: SimplifyMapperData::new(),
            linearize_regs: LinearizeRegsData::new(),
        }
    }

    pub fn linearize_regs(&self) -> &LinearizeRegsData { &self.linearize_regs }
    pub fn linearize_regs_mut(&mut self) -> &mut LinearizeRegsData { &mut self.linearize_regs }
    pub fn simplify_mapper(&self) -> &SimplifyMapperData { &self.simplify_mapper }
    pub fn debug_name(&self) -> &str { &self.debug_name }
    pub fn regalloc(&self) -> &RegisterAllocator { &self.regalloc }
    pub fn types(&self) -> &TypeModel { &self.types }
    pub fn types_mut(&mut self) -> &mut TypeModel { &mut self.types }
    pub fn typing(&mut self) -> &mut Typing { &mut self.typing }
    pub fn defstore(&self) -> &DefStore { &self.defstore }
    pub fn defstore_mut(&mut self) -> &mut DefStore { &mut self.defstore }
    pub fn codegen_regnames(&mut self) -> &mut CodeGenRegNames { &mut self.codegen_regnames }
}
