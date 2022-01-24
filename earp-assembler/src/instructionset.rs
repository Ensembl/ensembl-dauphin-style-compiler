use std::{collections::{HashMap, HashSet}, fmt::Display};
use crate::error::AssemblerError;

#[derive(Debug,Clone,PartialEq,Eq,Hash,PartialOrd, Ord)]
pub(crate) struct InstructionSetId(pub String,pub u64);

impl Display for InstructionSetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}/{}",self.0,self.1)
    }
}

#[derive(Debug,Clone,PartialEq)]
pub(crate) enum ArgType {
    Any,
    Jump,
    Register
}

#[derive(Debug,Clone,PartialEq)]
pub(crate) enum ArgSpec {
    Any,
    Specific(Vec<Vec<ArgType>>)
}

#[derive(Debug,Clone)]
pub(crate) struct InstructionSet {
    opcodes: HashMap<String,(u64,ArgSpec)>,
    opcodes_used: HashSet<u64>,
    identifier: InstructionSetId
}

impl InstructionSet {
    pub(crate) fn new(identifier: &InstructionSetId) -> InstructionSet {
        InstructionSet {
            opcodes: HashMap::new(),
            opcodes_used: HashSet::new(),
            identifier: identifier.clone()
        }
    }

    pub(crate) fn add(&mut self, name: &str, opcode: u64, argspecs: ArgSpec) -> Result<(),AssemblerError> {
        if self.opcodes.contains_key(name) || self.opcodes_used.contains(&opcode) {
            return Err(AssemblerError::OpcodeInUse(name.to_string()))
        }
        self.opcodes.insert(name.to_string(),(opcode,argspecs));
        self.opcodes_used.insert(opcode);
        Ok(())
    }

    pub(crate) fn merge(&mut self, other: &InstructionSet) -> Result<(),AssemblerError> {
        for (name,(opcode,argspecs)) in other.opcodes.iter() {
            self.add(name,*opcode,argspecs.clone())?;
        }
        Ok(())
    }

    pub(crate) fn identifier(&self) -> &InstructionSetId { &self.identifier }

    pub(crate) fn next_opcode(&self) -> u64 {
        self.opcodes.iter().map(|(_,(v,_))| *v+1).max().unwrap_or(0)
    }

    #[cfg(test)]
    pub(crate) fn opcodes(&self) -> impl Iterator<Item=(&str,u64)> {
        self.opcodes.iter().map(|(k,(v,_))| (k.as_str(),*v))
    }

    pub(crate) fn lookup(&self, opcode: &str) -> Option<u64> {
        self.opcodes.get(opcode).map(|(v,_)| v).cloned()
    }

    pub(crate) fn lookup_argspec(&self, opcode: &str) -> Option<ArgSpec> {
        self.opcodes.get(opcode).map(|(_,v)| v).cloned()
    }
}

#[cfg(test)]
mod test {
    use crate::error::AssemblerError;
    use crate::instructionset::{InstructionSetId, ArgSpec, ArgType};
    use crate::opcodemap::load_opcode_map;
    use crate::testutil::{ no_error, yes_error };

    #[test]
    fn instruction_set_smoke() {
        let standard = no_error(load_opcode_map(include_str!("test/test.map")));
        let ids = standard.iter().map(|x|x.identifier().clone()).collect::<Vec<_>>();
        assert!(ids.contains(&InstructionSetId("std".to_string(),0)));
        assert!(ids.contains(&InstructionSetId("std".to_string(),1)));
        assert!(!ids.contains(&InstructionSetId("silly".to_string(),0)));
        for set in &standard {
            if set.identifier() == &InstructionSetId("std".to_string(),0) {
                assert_eq!(Some(0),set.lookup("copy"));
                assert_eq!(None,set.lookup("silly"));
                let mut goto = None;
                for (name,opcode) in set.opcodes() {
                    if name == "goto" {
                        goto = Some(opcode);
                    }
                }
                assert_eq!(Some(1),goto);
                assert_eq!(6,set.next_opcode());
            }
            if set.identifier() == &InstructionSetId("std".to_string(),1) {
                assert_eq!(Some(0),set.lookup("copy"));
                assert_eq!(Some(6),set.lookup("more"));                
                assert_eq!(7,set.next_opcode());
            }
        }
    }

    #[test]
    fn set_gap_test() {
        let standard = no_error(load_opcode_map(include_str!("test/instructionset/gap.map")));
        let mut found = false;
        for set in &standard {
            if set.identifier() == &InstructionSetId("std".to_string(),0) {
                assert_eq!(6,set.next_opcode());
                found = true;
            }
        }
        assert!(found);
    }

    #[test]
    fn instruction_in_use() {
        let in_use = yes_error(load_opcode_map(include_str!("test/instructionset/in-use.map")));
        let e = in_use.to_string();
        assert!(e.contains("Already In Use"));
        assert!(e.contains("push"));
        match in_use {
            AssemblerError::BadOpcodeMap(_) => {},
            _ => assert!(false)            
        }
    }

    #[test]
    fn instruction_number_in_use() {
        let in_use = yes_error(load_opcode_map(include_str!("test/instructionset/number-in-use.map")));
        let e = in_use.to_string();
        assert!(e.contains("Already In Use"));
        assert!(e.contains("halt"));
        match in_use {
            AssemblerError::BadOpcodeMap(_) => {},
            _ => assert!(false)            
        }
    }

    #[test]
    fn argspec_smoke() {
        let standard = no_error(load_opcode_map(include_str!("test/instructionset/argspec.map")));
        let ids = standard.iter().map(|x|x.identifier().clone()).collect::<Vec<_>>();
        assert!(ids.contains(&InstructionSetId("std".to_string(),0)));
        for set in &standard {
            if set.identifier() == &InstructionSetId("std".to_string(),0) {
                assert_eq!(Some(ArgSpec::Specific(vec![vec![ArgType::Register,ArgType::Any]])),set.lookup_argspec("copy"));
                assert_eq!(Some(ArgSpec::Any),set.lookup_argspec("goto"));
                assert_eq!(Some(ArgSpec::Specific(vec![
                        vec![ArgType::Register,ArgType::Any],
                        vec![ArgType::Jump,ArgType::Any],
                        vec![],
                    ])),set.lookup_argspec("push"));
                assert_eq!(Some(ArgSpec::Any),set.lookup_argspec("pop"));                
                assert_eq!(Some(ArgSpec::Specific(vec![
                        vec![ArgType::Jump,ArgType::Any],
                    ])),set.lookup_argspec("if"));
                assert_eq!(Some(ArgSpec::Specific(vec![
                        vec![],
                    ])),set.lookup_argspec("halt"));
            }
        }
    }
}
