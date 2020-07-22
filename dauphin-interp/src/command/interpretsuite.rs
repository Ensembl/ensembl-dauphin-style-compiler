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
use std::collections::HashMap;
use std::rc::Rc;
use serde_cbor::Value as CborValue;
use crate::command::{ CommandTypeId, CommandDeserializer, CommandSetId, Deserializer, InterpLibRegister, OpcodeMapping, CommandSetVerifier };
use crate::runtime::{ PayloadFactory };
use crate::util::cbor::{ cbor_array, cbor_int };
use crate::util::{ DauphinError };

pub struct CommandInterpretSuite {
    store: Deserializer,
    offset_to_command: HashMap<(CommandSetId,u32),CommandTypeId>,
    opcode_mapper: OpcodeMapping,
    minors: HashMap<(String,u32),u32>,
    verifier: CommandSetVerifier,
    payloads: HashMap<(String,String),Rc<Box<dyn PayloadFactory>>>
}

impl CommandInterpretSuite {
    pub fn new() -> CommandInterpretSuite {
        CommandInterpretSuite {
            opcode_mapper: OpcodeMapping::new(),
            offset_to_command: HashMap::new(),
            store: Deserializer::new(),
            minors: HashMap::new(),
            verifier: CommandSetVerifier::new(),
            payloads: HashMap::new()
        }
    }

    fn register_real(&mut self, mut set: InterpLibRegister) -> anyhow::Result<()> {
        let sid = set.id().clone();
        let version = sid.version();
        self.minors.insert((sid.name().to_string(),version.0),version.1);
        for ds in set.drain_commands().drain(..) {
            if let Some((opcode,_)) = ds.get_opcode_len()? {
                let cid = self.store.add(ds);
                self.offset_to_command.insert((sid.clone(),opcode),cid.clone());
                self.opcode_mapper.add_opcode(&sid,opcode);
            }
        }
        for (k,p) in set.drain_payloads().drain() {
            self.payloads.insert(k,p);
        }
        self.verifier.register2(&sid)?;
        self.opcode_mapper.recalculate();
        Ok(())
    }

    pub fn register(&mut self,set: InterpLibRegister) -> anyhow::Result<()> {
        let name = set.id().name().to_string();
        self.register_real(set).with_context(|| format!("while registering {}",name))
    }

    pub fn copy_payloads(&self) -> HashMap<(String,String),Rc<Box<dyn PayloadFactory>>> {
        self.payloads.clone()
    }

    pub fn opcode_mapper(&self) -> &OpcodeMapping { &self.opcode_mapper }

    pub fn adjust(&mut self, cbor: &CborValue) -> anyhow::Result<()> {
        self.opcode_mapper = OpcodeMapping::deserialize(cbor)?;
        for (sid,_) in self.opcode_mapper.iter() {
            let name = sid.name().to_string();
            let version = sid.version();
            if let Some(stored_minor) = self.minors.get(&(name.clone(),version.0)) {
                if *stored_minor < version.1 {
                    return Err(DauphinError::integration(&format!("version of {}.{} too old. have {} need {}",name,version.0,stored_minor,version.1)));
                }
            } else {
                return Err(DauphinError::integration(&format!("missing command suite {}.{}",name,version.0)));
            }
        }
        Ok(())
    }

    pub fn get_deserializer(&self, real_opcode: u32) -> anyhow::Result<&Box<dyn CommandDeserializer>> {
        let (sid,offset) = self.opcode_mapper.decode_opcode(real_opcode)?;
        let cid = self.offset_to_command.get(&(sid,offset)).ok_or(DauphinError::malformed(&format!("Unknown opcode {}",real_opcode)))?;
        self.store.get(cid)
    }
}
