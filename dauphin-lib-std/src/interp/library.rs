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
    CommandSetId::new("std",(9,0),0x5419544B7434B16E)
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

pub struct RunDeserializer();

impl CommandDeserializer for RunDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((29,2))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(RunInterpCommand(Register::deserialize(&value[0])?,Register::deserialize(&value[1])?)))
    }
}

pub struct RunInterpCommand(Register,Register);

impl InterpCommand for RunInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let mut out = vec![];
        let datas = registers.get_indexes(&self.1)?;
        for len in datas.iter() {
            for i in 0..*len {
                out.push(i);
            }
        }
        registers.write(&self.0,InterpValue::Indexes(out));
        Ok(CommandResult::SyncResult())
    }
}

pub struct HaltDeserializer();

impl CommandDeserializer for HaltDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((30,1))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(HaltInterpCommand(Register::deserialize(&value[0])?)))
    }
}

pub struct HaltInterpCommand(Register);

impl InterpCommand for HaltInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let yn_in = registers.get_boolean(&self.0)?;
        let mut yn = false;
        for yn_this in yn_in.iter() {
            if *yn_this { yn = true; break; }
        }
        if yn {
            context.do_halt();
        }
        Ok(CommandResult::SyncResult())
    }
}

pub struct ExtractFilterDeserializer();

impl CommandDeserializer for ExtractFilterDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((27,7))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(ExtractFilterInterpCommand(Register::deserialize(&value[0])?,Register::deserialize(&value[1])?,
                                               Register::deserialize(&value[2])?,Register::deserialize(&value[3])?,
                                               Register::deserialize(&value[4])?,Register::deserialize(&value[5])?,
                                               Register::deserialize(&value[6])?)))
    }
}

fn extract_filter(values: &mut Vec<usize>, source_indexes: &mut Vec<usize>, range_indexes: &mut Vec<usize>,
                  start: usize, end: usize, range_starts_and_ends: &[(usize,(usize,usize))], source_index: usize) {
    for (range_index,(range_start,range_end)) in range_starts_and_ends {
        let ixn_start = *range_start.max(&start);
        let ixn_end = *range_end.min(&end);
        if ixn_start < ixn_end {
            for pos in ixn_start..ixn_end {
                range_indexes.push(*range_index);
                values.push(pos);
                source_indexes.push(source_index);
            }
        }
    }
}

pub struct ExtractFilterInterpCommand(Register,Register,Register,Register,Register,Register,Register);

impl InterpCommand for ExtractFilterInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let starts = registers.get_indexes(&self.3)?;
        let ends = registers.get_indexes(&self.4)?;
        let range_starts = registers.get_indexes(&self.5)?;
        let range_ends = registers.get_indexes(&self.6)?;
        let mut values = vec![];
        let mut source_indexes = vec![];
        let mut range_indexes = vec![];
        let range_starts_and_ends = 
            range_starts.iter().cloned().zip(range_ends.iter().cycle().cloned()).enumerate().collect::<Vec<_>>();
        for (i,(start,end)) in starts.iter().zip(ends.iter().cycle()).enumerate() {
            extract_filter(&mut values, &mut source_indexes, &mut range_indexes,*start,*end,&range_starts_and_ends,i);
        }
        registers.write(&self.0,InterpValue::Indexes(values));
        registers.write(&self.1,InterpValue::Indexes(source_indexes));
        registers.write(&self.2,InterpValue::Indexes(range_indexes));
        Ok(CommandResult::SyncResult())
    }
}

pub struct SetDifferenceDeserializer();

impl CommandDeserializer for SetDifferenceDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((34,3))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(SetDifferenceInterpCommand(Register::deserialize(&value[0])?,Register::deserialize(&value[1])?,
                                               Register::deserialize(&value[2])?)))
    }
}

fn set_difference(a: &[usize], b: &[usize])  -> Vec<bool> {
    let mut out = vec![];
    let mut b = b.to_vec();
    b.sort();
    let mut a_iter = a.iter();
    let mut b_iter = b.iter();
    let mut b_next = b_iter.next();
    while let Some(a_value) = a_iter.next() {
        /* if there's b left, make sure it's not less than a */
        while let Some(b_value) = b_next {
            if b_value >= a_value { break; }
            b_next = b_iter.next();
        }
        let mut hit = true;
        /* skip any match */
        if let Some(b_value) = b_next {
            if a_value == b_value { hit = false; }
        }
        /* push value */
        out.push(hit);
    }
    out
}

pub struct SetDifferenceInterpCommand(Register,Register,Register);

impl InterpCommand for SetDifferenceInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let a = registers.get_indexes(&self.1)?;
        let b = registers.get_indexes(&self.2)?;
        let c = set_difference(&a,&b);
        registers.write(&self.0,InterpValue::Boolean(c));
        Ok(CommandResult::SyncResult())
    }
}

pub struct RulerIntervalDeserializer();

impl CommandDeserializer for RulerIntervalDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((31,3))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(RulerIntervalInterpCommand(Register::deserialize(&value[0])?,Register::deserialize(&value[1])?,
                                               Register::deserialize(&value[2])?)))
    }
}

fn ruler_interval(region: i64, max_points: i64) -> i64 {
    let mut b10_value = 1;
    for _ in 0..20 {
        for mul in &[1,2] {
            let value = b10_value*mul;
            if region / value < max_points {
                return value;
            }
        }
        b10_value *= 10;
    }
    b10_value
}

pub struct RulerIntervalInterpCommand(Register,Register,Register);

impl InterpCommand for RulerIntervalInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let region = registers.get_numbers(&self.1)?;
        let max_points = registers.get_numbers(&self.2)?;
        let mut out = vec![];
        for (region,max_points) in region.iter().zip(max_points.iter().cycle()) {
            out.push(ruler_interval(region.ceil() as i64,*max_points as i64) as f64);
        }
        registers.write(&self.0,InterpValue::Numbers(out));
        Ok(CommandResult::SyncResult())
    }
}

pub struct RulerMarkingsDeserializer();

impl CommandDeserializer for RulerMarkingsDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((32,4))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(RulerMarkingsInterpCommand(Register::deserialize(&value[0])?,Register::deserialize(&value[1])?,
                                               Register::deserialize(&value[2])?,Register::deserialize(&value[3])?)))
    }
}

fn ruler_markings(interval: i64, min: i64, max: i64) -> Vec<i64> {
    let mut out = vec![];
    let mut pos = min;
    if pos%interval != 0 { pos += interval - (pos%interval); }
    while pos < max {
        out.push(pos);
        pos += interval;
    }
    out
}

pub struct RulerMarkingsInterpCommand(Register,Register,Register,Register);

impl InterpCommand for RulerMarkingsInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let interval = registers.get_numbers(&self.1)?.iter().map(|x| *x as i64).min().unwrap_or(1);
        let min = registers.get_numbers(&self.2)?.iter().map(|x| *x as i64).min().unwrap_or(1);
        let max = registers.get_numbers(&self.3)?.iter().map(|x| *x as i64).max().unwrap_or(1);
        let out = ruler_markings(interval,min,max).iter().map(|x| *x as f64).collect();
        registers.write(&self.0,InterpValue::Numbers(out));
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
    set.push(SetDifferenceDeserializer());
    set.push(RunDeserializer());
    set.push(HaltDeserializer());
    set.push(RulerIntervalDeserializer());
    set.push(RulerMarkingsDeserializer());
    set
}
