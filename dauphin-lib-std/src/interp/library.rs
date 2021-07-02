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
use dauphin_interp::polymorphic;
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
    CommandSetId::new("std",(0,5),0xF333A5BF97DDBBC2)
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

pub struct ExtractFilterDeserializer();

impl CommandDeserializer for ExtractFilterDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((27,4))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(ExtractFilterInterpCommand(Register::deserialize(&value[0])?,Register::deserialize(&value[1])?,
                                               Register::deserialize(&value[2])?,Register::deserialize(&value[3])?)))
    }
}

fn interval_union(starts: &[usize], ends: &[usize]) -> Vec<(usize,usize,usize)> {
    let mut intervals = starts.iter().zip(ends.iter().cycle()).enumerate()
        .map(|(i,x)| (*x.0,*x.1,i))
        .collect::<Vec<_>>();
    intervals.sort();
    let mut out : Vec<(usize,usize,usize)> = vec![];
    for (start,end,index) in intervals {
        let len = out.len();
        if len>0 && start <= out[len-1].1 {
            out[len-1].1 = end;
        } else {
            out.push((start,end,index));
        }
    }
    out
}

fn extract_filter(src: &[usize], starts: &[usize], ends: &[usize]) -> Vec<usize> {
    let sentinel = starts.len();
    /* we can use a simple binary search after unioning intervals to ensure no overlap */
    let intervals = interval_union(starts,ends);
    let mut values = src.iter().cloned().enumerate().collect::<Vec<_>>();
    values.sort_by_key(|(_,value)| *value);
    let mut interval_iter = intervals.iter().peekable();
    let mut out = vec![];
    for (index,value) in values {
        loop {
            let mut advance = false;
            if let Some((_,peek_end,_)) = interval_iter.peek() {
                if *peek_end <= value { advance = true; }
            }
            if advance { interval_iter.next(); } else { break; }
        }
        let mut out_value = sentinel;
        if let Some((peek_start,_,index)) = interval_iter.peek() {
            if value >= *peek_start { out_value = *index; }
        }
        out.push((index,out_value));
    }
    out.sort_by_key(|(pos,_)| *pos);
    let out = out.drain(..).map(|x| x.1).collect();
    use web_sys::console;
    console::log_1(&format!("extract_filter({:?},{:?},{:?})={:?}",src,starts,ends,out).into());
    out
}

pub struct ExtractFilterInterpCommand(Register,Register,Register,Register);

impl InterpCommand for ExtractFilterInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let src = registers.get_indexes(&self.1)?;
        let range_starts = registers.get_indexes(&self.2)?;
        let range_ends = registers.get_indexes(&self.3)?;
        let dst = extract_filter(&src,&range_starts,&range_ends);
        registers.write(&self.0,InterpValue::Indexes(dst));
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
    set.push(ExtractFilterDeserializer());
    set
}
