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

use anyhow::{ anyhow as err, bail };
use std::cell::Ref;
use std::rc::Rc;
use dauphin_interp::command::{ InterpCommand, InterpLibRegister, CommandDeserializer, CommandResult };
use dauphin_interp::types::{ SharedVec, RegisterSignature, XStructure, RegisterVectorSource, VectorRegisters, to_xstructure };
use dauphin_interp::runtime::{ InterpContext, InterpValue, InterpNatural, Register, RegisterFile };
use dauphin_interp::util::DauphinError;
use dauphin_interp::util::cbor::cbor_array;
use serde_cbor::Value as CborValue;
use dauphin_interp::stream::Stream;

// XXX dedup
pub fn std_stream(context: &mut InterpContext) -> anyhow::Result<&mut Stream> {
    let p = context.payload("std","stream")?;
    Ok(p.as_any_mut().downcast_mut().ok_or_else(|| DauphinError::runtime("No stream context"))?)
}

fn print_simple(sv: &SharedVec, path: &[usize], first: usize) -> anyhow::Result<String> {
    let (data,offset) = vr_lookup_data(sv,path,first)?;
    Ok(match data.get_natural() {
        InterpNatural::Empty => "".to_string(),
        InterpNatural::Indexes => format!("{}",data.to_rc_indexes()?.0.get(offset).ok_or(err!("corrupt data 1"))?),
        InterpNatural::Numbers => format!("{}",data.to_rc_numbers()?.0.get(offset).ok_or(err!("corrupt data 2"))?),
        InterpNatural::Boolean => format!("{}",data.to_rc_boolean()?.0.get(offset).ok_or(err!("corrupt data 3"))?),
        InterpNatural::Strings => format!("\"{}\"",data.to_rc_strings()?.0.get(offset).ok_or(err!("corrupt data 4"))?),
        InterpNatural::Bytes => format!("\'{}\'",data.to_rc_bytes()?.0.get(offset).ok_or(err!("corrupt data 5"))?.iter().map(|x| format!("{:02x}",x)).collect::<Vec<_>>().join(""))
    })
}

fn vr_lookup_data(sv: &SharedVec, path: &[usize], first: usize) -> anyhow::Result<(Rc<InterpValue>,usize)> {
    let mut position = first;
    for (i,index) in path.iter().enumerate() {
        let offset_val = sv.get_offset(sv.depth()-1-i)?;
        if offset_val.len() == 0 { bail!("corrupt data 6") }
        position = offset_val.get(position%offset_val.len()).ok_or(err!("corrupt data 7"))? + index;
    }
    let data = sv.get_data().clone();
    let data_len = data.len();
    if data_len == 0 { bail!("corrupt data 8") }
    Ok((data,position%data_len))
}

fn vr_lookup_len(sv: &SharedVec, path: &[usize], first: usize) -> anyhow::Result<usize> {
    let mut position = first;
    for (i,index) in path.iter().enumerate() {
        let offset_val = sv.get_offset(sv.depth()-1-i)?;
        position = offset_val.get(position).ok_or(err!("corrupt data 10"))? + index;
    }
    let len_val = sv.get_length(sv.depth()-1-path.len())?;
    Ok(*len_val.get(position).ok_or(err!("corrupt data 9"))?)
}

fn longest(xs: &XStructure<SharedVec>) -> Ref<SharedVec> {
    xs.max(|sv| {
        if sv.depth() >0 {
            sv.get_length(sv.depth()-1).map(|x| x.len()).unwrap_or(0)
        } else {
            sv.get_data().len()
        }
    }, 0).unwrap_or_else(|| xs.any())
}

fn print(file: &RegisterFile, xs: &XStructure<SharedVec>, regs: &[Register], path: &[usize], first: usize) -> anyhow::Result<String> {
    Ok(match xs {
        XStructure::Vector(xs_inner) => {
            let sv = longest(xs);
            let len = vr_lookup_len(&sv,path,first)?;
            let mut out = vec![];
            for i in 0..len {
                let mut new_path = path.to_vec();
                new_path.push(i);
                out.push(print(file,xs_inner,regs,&new_path,first)?);
            }
            format!("[{}]",out.join(", "))
        },
        XStructure::Struct(id,kvs) => {
            let mut subs : Vec<String> = kvs.keys().cloned().collect();
            subs.sort();
            let kvs : Vec<(String,_)> = subs.drain(..).map(|k| (k.clone(),kvs.get(&k).unwrap().clone())).collect();
            let out = kvs.iter().map(|(name,xs_inner)| 
                Ok(format!("{}: {}",name,print(file,xs_inner,regs,path,first)?))
            ).collect::<anyhow::Result<Vec<_>>>()?;
            format!("{} {{ {} }}",id.to_string(),out.join(", "))
        },
        XStructure::Enum(id,order,kvs,disc) => {
            let (data,offset) = vr_lookup_data(&disc.borrow(),path,first)?;
            let disc_val = data.to_rc_indexes()?.0[offset];
            let inner_xs = kvs.get(&order[disc_val]).ok_or_else(|| DauphinError::internal(file!(),line!()))?;
            format!("{}:{} {}",id,order[disc_val],print(file,inner_xs,regs,path,first)?)
        },
        XStructure::Simple(sv) => print_simple(&sv.borrow(),path,first)?,
    })
}

pub struct PrintInterpCommand(Register,Register,Register);

impl InterpCommand for PrintInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers();
        let texts = registers.get_strings(&self.2)?;
        let yn = registers.get_boolean(&self.0)?;
        let mut yn_iter = yn.iter().cycle();
        let level = registers.get_indexes(&self.1)?;
        let mut level_iter = level.iter().cycle();
        for text in texts.iter() {
            let yn = yn_iter.next().unwrap();
            let level = level_iter.next().unwrap();
            if *yn {
                std_stream(context)?.add(*level as u8,text);
            }
        }
        Ok(CommandResult::SyncResult())
    }
}

pub struct FormatDeserializer();

impl CommandDeserializer for FormatDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((2,2))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        let regs = cbor_array(&value[0],0,true)?.iter().map(|x| Register::deserialize(x)).collect::<Result<_,_>>()?;
        let sig = RegisterSignature::deserialize(value[1],true)?;
        Ok(Box::new(FormatInterpCommand(regs,sig)))        
    }
}

pub struct FormatInterpCommand(Vec<Register>,RegisterSignature);

impl InterpCommand for FormatInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let xs = to_xstructure(&self.1[1])?;
        let vs = RegisterVectorSource::new(&self.0);
        let xs2 = xs.derive(&mut (|vr: &VectorRegisters| SharedVec::new(context,&vs,vr)))?;
        let sv = longest(&xs2);
        let num = if sv.depth() > 0 { sv.get_offset(sv.depth()-1)?.len() } else { sv.get_data().len() };
        let registers = context.registers_mut();
        let mut out = vec![];
        for i in 0..num {
            out.push(print(&registers,&xs2,&self.0,&vec![],i)?);
        }
        registers.write(&self.0[0],InterpValue::Strings(out));
        Ok(CommandResult::SyncResult())
    }
}

pub struct PrintDeserializer();

impl CommandDeserializer for PrintDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((14,3))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(PrintInterpCommand(Register::deserialize(value[0])?,Register::deserialize(value[1])?,Register::deserialize(value[2])?)))
    }
}
pub struct CommaFormatDeserializer();

impl CommandDeserializer for CommaFormatDeserializer {
    fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some((33,2))) }
    fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
        Ok(Box::new(CommaFormatInterpCommand(Register::deserialize(value[0])?,Register::deserialize(value[1])?)))
    }
}

fn format_number(number: f64) -> String {
    let mut number = number.round() as i64;
    let mut out = vec![];
    while number > 1000 {
        out.push(format!("{0:0<3}",number%1000));
        number /= 1000;
    }
    out.push(number.to_string());
    out.reverse();
    out.join(",")
}

pub struct CommaFormatInterpCommand(Register,Register);

impl InterpCommand for CommaFormatInterpCommand {
    fn execute(&self, context: &mut InterpContext) -> anyhow::Result<CommandResult> {
        let registers = context.registers_mut();
        let numbers = registers.get_numbers(&self.1)?;
        let mut out = vec![];
        for number in numbers.iter() {
            out.push(format_number(*number));
        }
        registers.write(&self.0,InterpValue::Strings(out));
        Ok(CommandResult::SyncResult())
    }
}

pub(super) fn library_print_commands_interp(set: &mut InterpLibRegister) {
    set.push(PrintDeserializer());
    set.push(FormatDeserializer());
    set.push(CommaFormatDeserializer());
}