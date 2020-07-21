use anyhow;
use chrono::{ Local, DateTime };
use std::collections::HashMap;
use serde_cbor::Value as CborValue;
use crate::command::ProgramMetadata;
use dauphin_interp::util::DauphinError;
use dauphin_interp::util::cbor::{ cbor_map, cbor_int, cbor_map_iter, cbor_string };

pub(super) const VERSION : u32 = 0;

pub struct MetaLink {
    metadata: HashMap<String,ProgramMetadata>
}

impl MetaLink {
    pub fn new(prog: &CborValue) -> anyhow::Result<MetaLink> {
        let mut out = MetaLink {
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

    pub fn ls(&self) -> Vec<String> {
        if self.metadata.len() == 0 { return vec![]; }
        let max_name_len = self.metadata.iter().map(|(_,v)| v.name().len()).max().unwrap();
        let mut out = vec![];
        out.push(format!("{:width$} {:12} {:24} {:>6} {:>6} {}",
            "name","user","ctime","instrs","blocks","notes",
            width = max_name_len
        ));
        for (_,program) in self.metadata.iter() {
            let now_local : DateTime<Local> = program.ctime().clone().into();
            out.push(format!("{:width$} {:12} {:24} {:>6} {:>6} {}",
                program.name(),program.user().unwrap_or("-"),
                now_local.format("%Y-%m-%d %H:%M:%S%.3f"),
                program.instr_count(),program.inter_count(),
                program.note().unwrap_or(""),
                width = max_name_len));
        }
        out
    }
}