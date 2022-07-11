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

use std::collections::HashMap;
use std::hash::Hash;

use dauphin_interp::command::{ CommandSetId, InterpCommand, CommandDeserializer, InterpLibRegister, CommandResult };
use dauphin_interp::runtime::{InterpContext, InterpValue, Register};
use dauphin_interp::util::DauphinError;
use dauphin_interp::util::templates::NoopDeserializer;
use peregrine_toolkit::{ApproxNumber, log};
use serde_cbor::Value as CborValue;
use super::eq::{ library_eq_command_interp };
use super::numops::{ library_numops_commands_interp };
use super::vector::{ library_vector_commands_interp };
use super::print::{ library_print_commands_interp };
use super::map::{ library_map_commands_interp };


pub fn std_id() -> CommandSetId {
    CommandSetId::new("std",(13,0),0xD2ABC6BB06DED86)
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

pub struct NthDeserializer();

impl CommandDeserializer for NthDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((27,2))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(NthInterpCommand(Register::deserialize(&value[0])?,Register::deserialize(&value[1])?)))
    }
}

fn nth_command<T: Eq+Hash>(src: &[T]) -> Vec<usize> {
    let mut state = HashMap::new();
    let mut out = vec![];
    for item in src {
        if !state.contains_key(&item) {
            state.insert(item.clone(),0);
        }
        let value = state.get_mut(&item).unwrap();
        out.push(*value);
        *value += 1;
    }
    out
}

pub struct NthInterpCommand(Register,Register);

impl InterpCommand for NthInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let src = registers.get(&self.1).borrow().get_shared()?;
        let natural = src.get_natural();
        let out = match natural {
            dauphin_interp::runtime::InterpNatural::Strings => {
                nth_command(&src.to_rc_strings()?.0)
            },
            dauphin_interp::runtime::InterpNatural::Empty => { vec![] },
            dauphin_interp::runtime::InterpNatural::Numbers => {
                let values = src.to_rc_numbers()?.0.clone().iter().map(|x| ApproxNumber(*x,6)).collect::<Vec<_>>();
                nth_command(&values)
            },
            dauphin_interp::runtime::InterpNatural::Indexes => {
                nth_command(&src.to_rc_indexes()?.0)
            },
            dauphin_interp::runtime::InterpNatural::Boolean => {
                nth_command(&src.to_rc_boolean()?.0)
            },
            dauphin_interp::runtime::InterpNatural::Bytes => {
                (0..src.len()).collect::<Vec<_>>()
            },
        };

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

pub struct GapsDeserializer();

impl CommandDeserializer for GapsDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((35,8))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(GapsInterpCommand(Register::deserialize(&value[0])?,Register::deserialize(&value[1])?,
                                               Register::deserialize(&value[2])?,Register::deserialize(&value[3])?,
                                               Register::deserialize(&value[4])?,Register::deserialize(&value[5])?,
                                               Register::deserialize(&value[6])?,Register::deserialize(&value[7])?)))
    }
}

fn process_gap(out: &mut Vec<(f64,f64)>, start: f64, end: f64, gap_start: Option<f64>, gap_end: Option<f64>) {
    let gap_end = if let Some(gap_end) = gap_end {
        if gap_end <= start { return; } // gap has end and ends before start
        gap_end.min(end) // gap has end after our start, trim end if necessary
    } else {
        end // gap is endless on right, trim
    };
    let gap_start = if let Some(gap_start) = gap_start {
        if gap_start >= end { return; } // gap has start and starts after end
        gap_start.max(start) // gap has start after out start, trim start if necessary
    } else {
        start // gap is endless on left, trim
    };
    if gap_end >= gap_start {
        out.push((gap_start,gap_end));
    }
}

fn gaps_one(start: f64, end: f64, mut blocks: Vec<(f64,f64)>) -> Vec<(f64,f64)> {
    blocks.sort_by_key(|(a,b)| a.partial_cmp(b).unwrap());
    /* Call our block start ends (a0,b0), (a1,b1), (a2,b2) etc (an,bn)
     * Our gaps are then (-INF,a0), (b0,a1), (b1,a2),  (bn,+INF).
     * Pass these to process_gap().
     */
     let mut out = vec![];
    let mut prev_start = None;
    for (block_start,block_end) in blocks.iter() {
        process_gap(&mut out, start,end,prev_start,Some(*block_start));
        prev_start = Some(*block_end);
    }
    process_gap(&mut out, start,end,prev_start,None);
    out
}

fn gaps(starts: &[f64], ends: &[f64], block_starts: &[f64], block_ends: &[f64], block_indexes: &[usize]) -> (Vec<f64>,Vec<f64>,Vec<usize>) {
    // TODO check lens etc
    /* Assemble the blocks by index */
    let mut blocks = vec![vec![];starts.len()];
    for (index,(start,end)) in block_indexes.iter().zip(block_starts.iter().zip(block_ends.iter())) {
        blocks[*index].push((*start,*end));
    }
    /* Process each separately */
    let mut out_start = vec![];
    let mut out_end = vec![];
    let mut out_index = vec![];
    for (index,((start,end),blocks)) in starts.iter().zip(ends.iter()).zip(blocks.drain(..)).enumerate() {
       for (gap_start, gap_end) in gaps_one(*start,*end,blocks) {
           out_start.push(gap_start);
           out_end.push(gap_end);
           out_index.push(index);
       }
    }
    (out_start,out_end,out_index)
}

pub struct GapsInterpCommand(Register,Register,Register,Register,Register,Register,Register,Register);

impl InterpCommand for GapsInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let starts = registers.get_numbers(&self.3)?;
        let ends = registers.get_numbers(&self.4)?;
        let block_starts = registers.get_numbers(&self.5)?;
        let block_ends = registers.get_numbers(&self.6)?;
        let block_indexes = registers.get_indexes(&self.7)?;
        let (out_start,out_end,out_index) = gaps(&starts,&ends,&block_starts,&block_ends,&block_indexes);
        registers.write(&self.0,InterpValue::Numbers(out_start));
        registers.write(&self.1,InterpValue::Numbers(out_end));
        registers.write(&self.2,InterpValue::Indexes(out_index));
        Ok(CommandResult::SyncResult())
    }
}

pub struct RangeDeserializer();

impl CommandDeserializer for RangeDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((36,4))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(RangeInterpCommand(Register::deserialize(&value[0])?,Register::deserialize(&value[1])?,
                                               Register::deserialize(&value[2])?,Register::deserialize(&value[3])?)))
    }
}

pub struct RangeInterpCommand(Register,Register,Register,Register);

impl InterpCommand for RangeInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let starts = registers.get_numbers(&self.2)?;
        let ends = registers.get_numbers(&self.3)?;
        let mut out_pos = vec![];
        let mut out_idx = vec![];
        for (index,(start,end)) in starts.iter().zip(ends.iter()).enumerate() {
            let start = start.round() as i64;
            let end = end.round() as i64;
            for pos in start..end {
                out_pos.push(pos as f64);
                out_idx.push(index);
            }
        }
        registers.write(&self.0,InterpValue::Numbers(out_pos));
        registers.write(&self.1,InterpValue::Indexes(out_idx));
        Ok(CommandResult::SyncResult())
    }
}

pub struct SplitCharactersDeserializer();

impl CommandDeserializer for SplitCharactersDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((37,4))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(SplitCharactersInterpCommand(Register::deserialize(&value[0])?,Register::deserialize(&value[1])?,
                                               Register::deserialize(&value[2])?,Register::deserialize(&value[3])?)))
    }
}

pub struct SplitCharactersInterpCommand(Register,Register,Register,Register);

impl InterpCommand for SplitCharactersInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let strings = registers.get_strings(&self.3)?;
        let mut out_data = vec![];
        let mut out_start = vec![];
        let mut out_len = vec![];
        for string in strings.iter() {
            let old_len = out_data.len();
            out_start.push(old_len);
            out_data.extend(string.chars().map(|x| x.to_string()));
            out_len.push(out_data.len()-old_len);
        }
        registers.write(&self.0,InterpValue::Strings(out_data));
        registers.write(&self.1,InterpValue::Indexes(out_start));
        registers.write(&self.2,InterpValue::Indexes(out_len));
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
    set.push(NthDeserializer());
    set.push(DerunDeserializer());
    set.push(GapsDeserializer());
    set.push(RangeDeserializer());
    set.push(SetDifferenceDeserializer());
    set.push(RunDeserializer());
    set.push(HaltDeserializer());
    set.push(RulerIntervalDeserializer());
    set.push(RulerMarkingsDeserializer());
    set.push(SplitCharactersDeserializer());
    set
}
