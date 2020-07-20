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

use crate::command::{ CommandDeserializer, InterpCommand };
use crate::runtime::{ InterpContext };
use crate::util::DauphinError;
use serde_cbor::Value as CborValue;

pub struct ErrorInterpCommand();

impl InterpCommand for ErrorInterpCommand {
    fn execute(&self, _context: &mut InterpContext) -> anyhow::Result<()> {
        Err(DauphinError::malformed("compile time command somehow ended up in binary"))
    }
}

pub struct NoopInterpCommand();

impl InterpCommand for NoopInterpCommand {
    fn execute(&self, _context: &mut InterpContext) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct ErrorDeserializer();

impl CommandDeserializer for ErrorDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> {
        Ok(None)
    }
    fn deserialize(&self, _opcode: u32, _value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Err(DauphinError::malformed("compile time command somehow ended up in binary"))
    }
}

pub struct NoopDeserializer(pub u32);

impl CommandDeserializer for NoopDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> {
        Ok(Some((self.0,0)))
    }
    fn deserialize(&self, _opcode: u32, _value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(NoopInterpCommand()))
    }
}
