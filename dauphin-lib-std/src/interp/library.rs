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

use dauphin_interp::command::{ CommandSetId, InterpCommand, CommandDeserializer, InterpLibRegister, CommandResult };
use dauphin_interp::runtime::{InterpContext, InterpValue, Register};
use dauphin_interp::util::DauphinError;
use dauphin_interp::util::templates::NoopDeserializer;
use serde_cbor::Value as CborValue;
use super::eq::{ library_eq_command_interp };
use super::numops::{ library_numops_commands_interp };
use super::vector::{ library_vector_commands_interp };
use super::print::{ library_print_commands_interp };
use super::map::{ library_map_commands_interp };

pub fn std_id() -> CommandSetId {
    CommandSetId::new("std",(0,4),0x44F0AA9C639A3211)
}

pub struct AssertDeserializer();

impl CommandDeserializer for AssertDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((4,2))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(AssertInterpCommand(Register::deserialize(&value[0])?,Register::deserialize(&value[1])?)))
    }
}

pub struct AssertInterpCommand(Register,Register);

impl InterpCommand for AssertInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let a = &registers.get_boolean(&self.0)?;
        let b = &registers.get_boolean(&self.1)?;
        for i in 0..a.len() {
            if a[i] != b[i%b.len()] {
                return Err(DauphinError::runtime(&format!("assertion failed index={}!",i)));
            }
        }
        Ok(CommandResult::SyncResult())
    }
}

pub struct BytesToBoolDeserializer();

impl CommandDeserializer for BytesToBoolDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((25,2))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(BytesToBoolInterpCommand(Register::deserialize(&value[0])?,Register::deserialize(&value[1])?)))
    }
}

pub struct BytesToBoolInterpCommand(Register,Register);

impl InterpCommand for BytesToBoolInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let mut bools = vec![];
        let datas = registers.get_bytes(&self.1)?;
        for data in datas.iter() {
            bools.extend(data.iter().map(|x| *x!=0));
        }
        registers.write(&self.0,InterpValue::Boolean(bools));
        Ok(CommandResult::SyncResult())
    }
}

pub struct DerunDeserializer();

impl CommandDeserializer for DerunDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((26,2))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(DerunInterpCommand(Register::deserialize(&value[0])?,Register::deserialize(&value[1])?)))
    }
}

pub struct DerunInterpCommand(Register,Register);

impl InterpCommand for DerunInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let mut out = vec![];
        let datas = registers.get_indexes(&self.1)?;
        let mut next = 0;
        for mul in datas.iter() {
            for _ in 0..*mul {
                out.push(next);
            }
            next += 1;
        }
        registers.write(&self.0,InterpValue::Indexes(out));
        Ok(CommandResult::SyncResult())
    }
}

pub fn make_std_interp() -> InterpLibRegister {
    let mut set = InterpLibRegister::new(&std_id());
    library_eq_command_interp(&mut set);
    set.push(AssertDeserializer());
    set.push(NoopDeserializer(13));
    library_print_commands_interp(&mut set);
    library_numops_commands_interp(&mut set);
    library_vector_commands_interp(&mut set);
    library_map_commands_interp(&mut set);
    set.push(BytesToBoolDeserializer());
    set.push(DerunDeserializer());
    set
}
