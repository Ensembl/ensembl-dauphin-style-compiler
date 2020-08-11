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

use dauphin_compile::command::{
    Command, CommandSchema, CommandType, CommandTrigger, PreImageOutcome, Instruction, InstructionType, TimeTrial
};
use dauphin_compile::model::PreImageContext;
use dauphin_interp::command::InterpCommand;
use dauphin_interp::runtime::Register;
use dauphin_interp::types::{ FullType, RegisterSignature };
use dauphin_interp::util::DauphinError;
use dauphin_compile::util::{ vector_add_instrs, vector_append_data, vector_append_indexes, vector_register_copy_instrs };
use serde_cbor::Value as CborValue;
use super::library::std;

fn extend_real(out: &mut Vec<Instruction>, context: &mut PreImageContext, dst: &FullType, src: &FullType, copies_reg: Option<&Register>, regs: &Vec<Register>) -> anyhow::Result<()> {
    /* setup special constants zero and one */
    let one = context.new_register();
    out.push(Instruction::new(InstructionType::Const(vec![1]),vec![one]));
    let zero = context.new_register();
    out.push(Instruction::new(InstructionType::Const(vec![0]),vec![zero]));
    /* for every vr of each complex type ... */
    for (z,b) in dst.iter().zip(src.iter()) {
        let depth = z.1.depth();
        if depth > 0 {
            /* ... if it's a vector, we need to know the length of the penultimate layer in
             * the OUTPUT prior to extending because we are going to be adding to the end of
             * it and so we need to offset the offset of our ultimate layer by that amount.
             */
            let start = context.new_register();
            let reg_off = if depth > 1 {
                z.1.offset_pos(depth-2)?
            } else {
                z.1.data_pos()
            };
            out.push(Instruction::new(InstructionType::Length,vec![start,regs[reg_off]]));
            /* calculate top-level stride ... */
            let (stride,copies) = if let Some(copies_reg) = copies_reg {
                /* ... if we don't know how many copies, we need to measure source for stride */
                let size_reg = context.new_register();
                let b_off = b.1.offset_pos(depth-1)?;
                out.push(Instruction::new(InstructionType::Length,vec![size_reg,regs[b_off]]));
                (size_reg,*copies_reg)
            } else {
                /* ... but if we know we need just one copy (extend) stride can be zero */
                (zero,one)
            };
            /* everything except the top layer can be pushed normally */
            out.append(&mut vector_add_instrs(context,z.1,b.1,&copies,regs)?);
            /* push top layer... */
            /* ... push one copy of the top offset reg of b onto z with offset start and stride zero */
            out.push(vector_append_indexes(z.1,b.1,&start,&stride,&copies,regs,depth-1)?);
            /* ... push one copy of the top lenght reg of b onto z with given reg containing zero */
            out.push(vector_append_indexes(z.1,b.1,&zero,&zero,&copies,regs,depth-1)?);
        } else {
            /* ... if it's not a vector, simply append one copy of it */
            out.push(vector_append_data(z.1,b.1,&copies_reg.unwrap_or(&one),&regs)?);
        }
    }
    Ok(())
}

fn extend(context: &mut PreImageContext, sig: &RegisterSignature, regs: &Vec<Register>) -> anyhow::Result<Vec<Instruction>> {
    let mut out = vec![];
    /* a is copied with just a series of copies */
    for (vr_z,vr_a) in sig[0].iter().zip(sig[1].iter()) {
        out.append(&mut vector_register_copy_instrs(&vr_z.1,&vr_a.1,regs)?);
    }
    extend_real(&mut out,context,&sig[0],&sig[2],None,regs)?;
    Ok(out)
}

fn repeat(context: &mut PreImageContext, sig: &RegisterSignature, regs: &Vec<Register>) -> anyhow::Result<Vec<Instruction>> {
    let mut out = vec![];
    for vr_z in sig[0].iter() {
        for r in vr_z.1.all_registers() {
            out.push(Instruction::new(InstructionType::Nil,vec![regs[r]]));
        }
    }
    let count = regs[sig[1].iter().next().unwrap().1.data_pos()];
    extend_real(&mut out,context,&sig[0],&sig[2],Some(&count),regs)?;
    Ok(out)
}

pub struct ExtendCommandType(Option<TimeTrial>);

impl ExtendCommandType {
    pub fn new() -> ExtendCommandType { ExtendCommandType(None) }
}

impl CommandType for ExtendCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 0,
            trigger: CommandTrigger::Command(std("extend"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            Ok(Box::new(ExtendCommand(sig.clone(),it.regs.to_vec(),self.0.clone())))
        } else {
            Err(DauphinError::malformed("unexpected instruction"))
        }
    }    
}

pub struct ExtendCommand(RegisterSignature,Vec<Register>,Option<TimeTrial>);

impl Command for ExtendCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(None)
    }

    fn preimage(&self, context: &mut PreImageContext, _ic: Option<Box<dyn InterpCommand>>) -> anyhow::Result<PreImageOutcome> {
        Ok(PreImageOutcome::Replace(extend(context,&self.0,&self.1)?))
    }
}

pub struct RepeatCommandType(Option<TimeTrial>);

impl RepeatCommandType {
    pub fn new() -> RepeatCommandType { RepeatCommandType(None) }
}

impl CommandType for RepeatCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 0,
            trigger: CommandTrigger::Command(std("repeat"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            Ok(Box::new(RepeatCommand(sig.clone(),it.regs.to_vec(),self.0.clone())))
        } else {
            Err(DauphinError::malformed("unexpected instruction"))
        }
    }    
}

pub struct RepeatCommand(RegisterSignature,Vec<Register>,Option<TimeTrial>);

impl Command for RepeatCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(None)
    }

    fn preimage(&self, context: &mut PreImageContext, _ic: Option<Box<dyn InterpCommand>>) -> anyhow::Result<PreImageOutcome> {
        Ok(PreImageOutcome::Replace(repeat(context,&self.0,&self.1)?))
    }
}
