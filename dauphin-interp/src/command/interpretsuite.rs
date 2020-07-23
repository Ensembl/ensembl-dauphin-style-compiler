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
use std::cell::RefCell;
use serde_cbor::Value as CborValue;
use crate::command::{ CommandTypeId, CommandDeserializer, CommandSetId, Deserializer, InterpCommand, InterpLibRegister, OpcodeMapping, CommandSetVerifier };
use crate::runtime::{ PayloadFactory };
use crate::util::cbor::{ cbor_array, cbor_int };
use crate::util::{ DauphinError };

#[derive(Clone)]
pub struct CommandInterpretSuite {
    store: Rc<RefCell<Deserializer>>,
    offset_to_command: HashMap<(CommandSetId,u32),CommandTypeId>,
    opcode_mapper: OpcodeMapping,
    minors: HashMap<(String,u32),CommandSetId>,
    verifier: CommandSetVerifier,
    payloads: HashMap<(String,String),Rc<dyn PayloadFactory>>
}

impl CommandInterpretSuite {
    pub fn new() -> CommandInterpretSuite {
        CommandInterpretSuite {
            opcode_mapper: OpcodeMapping::new(),
            offset_to_command: HashMap::new(),
            store: Rc::new(RefCell::new(Deserializer::new())),
            minors: HashMap::new(),
            verifier: CommandSetVerifier::new(),
            payloads: HashMap::new()
        }
    }

    fn register_real(&mut self, mut set: InterpLibRegister) -> anyhow::Result<()> {
        let sid = set.id().clone();
        let version = sid.version();
        self.minors.insert((sid.name().to_string(),version.0),sid.clone());
        for ds in set.drain_commands().drain(..) {
            if let Some((opcode,_)) = ds.get_opcode_len()? {
                let cid = self.store.borrow_mut().add(ds);
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

    pub fn copy_payloads(&self) -> HashMap<(String,String),Rc<dyn PayloadFactory>> {
        self.payloads.clone()
    }

    pub fn default_opcode_mapper(&self) -> &OpcodeMapping { &self.opcode_mapper }
    pub(super) fn offset_to_command(&self, set: &CommandSetId, offset: u32) -> anyhow::Result<&CommandTypeId> {
        Ok(self.offset_to_command.get(&(set.clone(),offset)).ok_or(DauphinError::malformed(&format!("Unknown opcode {} offset {}",set,offset)))?)
    }

    pub fn get_opcode_len(&self, cid: &CommandTypeId) -> anyhow::Result<Option<(u32,usize)>> {
        Ok(self.store.borrow().get(cid)?.get_opcode_len()?)
    }

    // TODO real_opcode to deserialize why?
    pub fn deserialize(&self, cid: &CommandTypeId, real_opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        self.store.borrow().get(cid)?.deserialize(real_opcode,value)
    }

    pub fn prog_to_stored_set(&self, prog: &CommandSetId) -> anyhow::Result<CommandSetId> {
        Ok(self.minors.get(&(prog.name().to_string(),prog.version().0)).ok_or(DauphinError::malformed(&format!("Unknown library {}",prog)))?.clone())
    }
}
