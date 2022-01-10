use std::{collections::{HashMap, HashSet}, fmt::Display};
use crate::error::EarpAssemblerError;

#[derive(Debug,Clone,PartialEq,Eq,Hash,PartialOrd, Ord)]
pub(crate) struct EarpInstructionSetIdentifier(pub String,pub u64);

impl Display for EarpInstructionSetIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}/{}",self.0,self.1)
    }
}

#[derive(Debug,Clone)]
pub(crate) struct EarpInstructionSet {
    opcodes: HashMap<String,u64>,
    opcodes_used: HashSet<u64>,
    identifier: EarpInstructionSetIdentifier
}

impl EarpInstructionSet {
    pub(crate) fn new(identifier: &EarpInstructionSetIdentifier) -> EarpInstructionSet {
        EarpInstructionSet {
            opcodes: HashMap::new(),
            opcodes_used: HashSet::new(),
            identifier: identifier.clone()
        }
    }

    pub(crate) fn add(&mut self, name: &str, opcode: u64) -> Result<(),EarpAssemblerError> {
        if self.opcodes.contains_key(name) || self.opcodes_used.contains(&opcode) {
            return Err(EarpAssemblerError::OpcodeInUse(name.to_string()))
        }
        self.opcodes.insert(name.to_string(),opcode);
        self.opcodes_used.insert(opcode);
        Ok(())
    }

    pub(crate) fn merge(&mut self, other: &EarpInstructionSet) -> Result<(),EarpAssemblerError> {
        for (name,opcode) in other.opcodes() {
            self.add(name,opcode)?;
        }
        Ok(())
    }

    pub(crate) fn identifier(&self) -> &EarpInstructionSetIdentifier { &self.identifier }

    pub(crate) fn next_opcode(&self) -> u64 {
        self.opcodes.iter().map(|(_,v)| *v+1).max().unwrap_or(0)
    }

    pub(crate) fn opcodes(&self) -> impl Iterator<Item=(&str,u64)> {
        self.opcodes.iter().map(|(k,v)| (k.as_str(),*v))
    }

    pub(crate) fn lookup(&self, opcode: &str) -> Option<u64> {
        self.opcodes.get(opcode).cloned()
    }
}

#[cfg(test)]
mod test {
    use crate::error::EarpAssemblerError;
    use crate::instructionset::{EarpInstructionSetIdentifier};
    use crate::opcodemap::load_opcode_map;
    use crate::testutil::{ no_error, yes_error };

    #[test]
    fn instruction_set_smoke() {
        let standard = no_error(load_opcode_map(include_str!("test/test.map")));
        let ids = standard.iter().map(|x|x.identifier().clone()).collect::<Vec<_>>();
        assert!(ids.contains(&EarpInstructionSetIdentifier("std".to_string(),0)));
        assert!(ids.contains(&EarpInstructionSetIdentifier("std".to_string(),1)));
        assert!(!ids.contains(&EarpInstructionSetIdentifier("silly".to_string(),0)));
        for set in &standard {
            if set.identifier() == &EarpInstructionSetIdentifier("std".to_string(),0) {
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
            if set.identifier() == &EarpInstructionSetIdentifier("std".to_string(),1) {
                assert_eq!(Some(0),set.lookup("copy"));
                assert_eq!(Some(6),set.lookup("more"));                
                assert_eq!(7,set.next_opcode());
            }
        }
    }

    #[test]
    fn set_gap_test() {
        let standard = no_error(load_opcode_map(include_str!("test/test-gap.map")));
        let mut found = false;
        for set in &standard {
            if set.identifier() == &EarpInstructionSetIdentifier("std".to_string(),0) {
                assert_eq!(6,set.next_opcode());
                found = true;
            }
        }
        assert!(found);
    }

    #[test]
    fn instruction_in_use() {
        let in_use = yes_error(load_opcode_map(include_str!("test/in-use.map")));
        let e = in_use.to_string();
        assert!(e.contains("Already In Use"));
        assert!(e.contains("push"));
        match in_use {
            EarpAssemblerError::BadOpcodeMap(_) => {},
            _ => assert!(false)            
        }
    }

    #[test]
    fn instruction_number_in_use() {
        let in_use = yes_error(load_opcode_map(include_str!("test/number-in-use.map")));
        let e = in_use.to_string();
        assert!(e.contains("Already In Use"));
        assert!(e.contains("halt"));
        match in_use {
            EarpAssemblerError::BadOpcodeMap(_) => {},
            _ => assert!(false)            
        }
    }
}
