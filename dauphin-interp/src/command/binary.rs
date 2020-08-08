use anyhow;
use std::collections::HashMap;
use serde_cbor::Value as CborValue;
use crate::command::ProgramMetadata;
use crate::util::DauphinError;
use crate::util::cbor::{ cbor_map, cbor_int, cbor_map_iter, cbor_string };

pub(super) const VERSION : u32 = 0;

pub struct Binary {
    metadata: HashMap<String,ProgramMetadata>
}

impl Binary {
    pub fn new(prog: &CborValue) -> anyhow::Result<Binary> {
        let mut out = Binary {
            metadata: HashMap::new()
        };
        out.load(prog)?;
        Ok(out)
    }

    fn load(&mut self, cbor: &CborValue) -> anyhow::Result<()> {
        let data = cbor_map(cbor,&["version","suite","programs"])?;
        let got_ver = cbor_int(data[0],None)? as u32;
        if got_ver != VERSION {
            return Err(DauphinError::integration(&format!("Incompatible code. got v{} understand v{}",got_ver,VERSION)));
        }
        for (name,program) in cbor_map_iter(data[2])? {
            let metadata = ProgramMetadata::deserialize(cbor_map(program,&["metadata"])?[0])?;
            self.metadata.insert(cbor_string(name)?,metadata);
        }
        Ok(())
    }

    pub fn get_metadata(&self, name: &str) -> Option<&ProgramMetadata> {
        self.metadata.get(name)
    }
}