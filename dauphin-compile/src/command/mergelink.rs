/* 
 *  See the NOTICE file distributed with this work for additional information
 *  regarding copyright ownership.
 *  
 *  Licensed under the Apache License, Version 2.0 (the "License"); you may 
 *  not use this file except in compliance with the License. You may obtain a
 *  copy of the License at http://www.apache.org/licenses/LICENSE-2.0
 *  
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS, WITHOUT 
 *  WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 */

use anyhow::{ self, Context };
use std::collections::BTreeMap;
use dauphin_interp::command::{ CommandInterpretSuite };
use dauphin_interp::util::DauphinError;
use dauphin_interp::util::cbor::{ cbor_take_map };
use crate::command::{ OpcodeRemapper, RemapperEndpoint, VERSION, CommandCompileSuite, CompilerLink };
use serde_cbor::Value as CborValue;

pub struct MergeLink {
    clink: CompilerLink,
    ocr: OpcodeRemapper,
    programs: BTreeMap<CborValue,CborValue>
}

impl MergeLink {
    pub fn new(ccs: CommandCompileSuite) -> MergeLink {
        let ocr = OpcodeRemapper::new(&ccs.opcode_mapper().clone());
        MergeLink {
            clink: CompilerLink::new(ccs),
            ocr,
            programs: BTreeMap::new()
        }
    }

    pub fn add_file(&mut self, data: CborValue) -> anyhow::Result<()> {
        let mut filedata = cbor_take_map(data)?;
        let programs = filedata.remove(&CborValue::Text("programs".to_string())).ok_or_else(|| DauphinError::malformed("no programs section"))?;
        let suite = filedata.get(&CborValue::Text("suite".to_string())).ok_or_else(|| DauphinError::malformed("no programs section"))?;
        let rme = RemapperEndpoint::new(&mut self.ocr,suite)?;
        for (name,program) in cbor_take_map(programs)? {
            let mut program = cbor_take_map(program)?;
            let commands = program.remove(&CborValue::Text("cmds".to_string())).ok_or_else(|| DauphinError::malformed("no cmds section"))?;
            let cmds = rme.remap_program(&self.clink,&self.ocr,&commands).context("remapping")?;
            program.insert(CborValue::Text("cmds".to_string()),cmds);
            self.programs.insert(name,CborValue::Map(program));
        }
        Ok(())
    }

    pub fn serialize(self) -> anyhow::Result<CborValue> {
        let mut out = BTreeMap::new();
        out.insert(CborValue::Text("version".to_string()),CborValue::Integer(VERSION as i128));
        out.insert(CborValue::Text("suite".to_string()),self.ocr.serialize());
        out.insert(CborValue::Text("programs".to_string()),CborValue::Map(self.programs));
        Ok(CborValue::Map(out))
    }
}
