use std::collections::HashMap;
use minicbor::{Decoder, Decode, decode::Error};

use crate::{core::error::EarpRuntimeError, runtime::instruction::Instruction, suite::suite::Suite};

use super::{toplevel::{ TopLevel, map_error, self }, resolver::Resolver};

#[cfg_attr(debug_assertions,derive(Debug))]
pub enum AssetData {
    String(String),
    Bytes(Vec<u8>)
}

const MAGIC_NUMBER : &str = "EARP0";

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct EarpFileReader {
    entry_points: HashMap<String,i64>,
    assets: HashMap<String,AssetData>,
    instructions: Vec<Instruction>
}

impl EarpFileReader {
    fn magic_check(top_level: &TopLevel) -> Result<(),EarpRuntimeError> {
        if let Some(magic_got) = &top_level.magic_got {
            if magic_got != MAGIC_NUMBER {
                Err(EarpRuntimeError::BadMagic(format!("Got {:?}, expected {:?}",magic_got,MAGIC_NUMBER)))
            } else {
                Ok(())
            }
        } else {
            Err(EarpRuntimeError::BadMagic(format!("Missing, expected {:?}",MAGIC_NUMBER)))
        }
    }

    fn resolve(suite: &Suite, top_level: &TopLevel) -> Result<Vec<Instruction>,EarpRuntimeError> {
        let resolver = Resolver::new(suite,&top_level.sets);
        let mut out = vec![];
        for (offset,operands) in &top_level.instructions {
            let command = resolver.lookup(*offset)?;
            out.push(Instruction::new(&command,operands));
        }
        Ok(out)
    }

    pub fn new(suite: &Suite, data: &[u8]) -> Result<EarpFileReader,EarpRuntimeError> {
        let mut decoder = Decoder::new(data);
        let top_level = map_error(TopLevel::decode(&mut decoder))?;
        EarpFileReader::magic_check(&top_level)?;
        let instructions = EarpFileReader::resolve(suite,&top_level)?;
        Ok(EarpFileReader {
            entry_points: top_level.entry_points,
            assets: top_level.assets,
            instructions
        })
    }
}
