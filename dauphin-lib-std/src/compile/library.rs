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

use anyhow;
use dauphin_compile::command::{
    Command, CommandSchema, CommandType, CommandTrigger, PreImageOutcome, PreImagePrepare, CompLibRegister, Instruction, InstructionType
};
use dauphin_compile::model::PreImageContext;
use dauphin_interp::command::{ InterpCommand, CommandSetId, Identifier };
use dauphin_interp::types::{ RegisterSignature };
use dauphin_interp::runtime::{ Register };
use dauphin_interp::util::DauphinError;
use serde_cbor::Value as CborValue;
use super::numops::{ library_numops_commands };
use super::eq::{ library_eq_command };
use super::assign::{ library_assign_commands };
use super::print::{ PrintCommandType, FormatCommandType, CommaFormatCommandType };
use super::vector::{ library_vector_commands };
use super::map::{ library_map_commands };
use crate::make_std_interp;

pub fn std_id() -> CommandSetId {
    CommandSetId::new("std",(13,0),0xD2ABC6BB06DED86)
}

pub(super) fn std(name: &str) -> Identifier {
    Identifier::new("std",name)
}

pub struct BytesToBoolCommandType();

impl CommandType for BytesToBoolCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 2,
            trigger: CommandTrigger::Command(std("bytes_to_bool"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            Ok(Box::new(BytesToBoolCommand(it.regs[0].clone(),it.regs[1].clone())))
        } else {
            Err(DauphinError::malformed("unexpected instruction"))
        }
    }
}

pub struct BytesToBoolCommand(Register,Register);

impl Command for BytesToBoolCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![self.0.serialize(),self.1.serialize()]))
    }
}

pub struct LenCommandType();

impl CommandType for LenCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 2,
            trigger: CommandTrigger::Command(std("len"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            Ok(Box::new(LenCommand(sig.clone(),it.regs.clone())))
        } else {
            Err(DauphinError::malformed("unexpected instruction"))
        }
    }
}

pub struct LenCommand(pub(crate) RegisterSignature, pub(crate) Vec<Register>);

impl Command for LenCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![CborValue::Array(self.1.iter().map(|x| x.serialize()).collect()),self.0.serialize(false)?]))
    }

    fn preimage(&self, context: &mut PreImageContext, _ic: Option<Box<dyn InterpCommand>>) -> anyhow::Result<PreImageOutcome> {
        if let Some((_,ass)) = &self.0[1].iter().next() {
            let reg = ass.length_pos(ass.depth()-1)?;
            if context.is_reg_valid(&self.1[reg]) && !context.is_last() {
                /* can execute now */
                context.context_mut().registers_mut().copy(&self.1[0],&self.1[reg])?;
                return Ok(PreImageOutcome::Constant(vec![self.1[0]]));
            } else {
                /* replace */
                return Ok(PreImageOutcome::Replace(vec![
                    Instruction::new(InstructionType::Copy,vec![self.1[0].clone(),self.1[reg].clone()])
                ]))
            }
        }
        /* should never happen! */
        Err(DauphinError::internal(file!(),line!()))
    }
}

pub struct DerunCommandType();

impl CommandType for DerunCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 2,
            trigger: CommandTrigger::Command(std("derun"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            Ok(Box::new(DerunCommand(it.regs[0].clone(),it.regs[1].clone())))
        } else {
            Err(DauphinError::malformed("unexpected instruction"))
        }
    }
}

pub struct DerunCommand(Register,Register);

impl Command for DerunCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![self.0.serialize(),self.1.serialize()]))
    }
}

pub struct RunCommandType();

impl CommandType for RunCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 2,
            trigger: CommandTrigger::Command(std("run"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            Ok(Box::new(RunCommand(it.regs[0].clone(),it.regs[1].clone())))
        } else {
            Err(DauphinError::malformed("unexpected instruction"))
        }
    }
}

pub struct RunCommand(Register,Register);

impl Command for RunCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![self.0.serialize(),self.1.serialize()]))
    }
}

pub struct NthCommandType();

impl CommandType for NthCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 2,
            trigger: CommandTrigger::Command(std("nth"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            Ok(Box::new(NthCommand(it.regs[0].clone(),it.regs[1].clone())))
        } else {
            Err(DauphinError::malformed("unexpected instruction"))
        }
    }
}

pub struct NthCommand(Register,Register);

impl Command for NthCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![self.0.serialize(),self.1.serialize()]))
    }
}


pub struct ExtractFilterCommandType();

impl CommandType for ExtractFilterCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 7,
            trigger: CommandTrigger::Command(std("extract_filter"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            Ok(Box::new(ExtractFilterCommand(
                it.regs[0].clone(),it.regs[1].clone(),it.regs[2].clone(),
                it.regs[3].clone(),it.regs[4].clone(),it.regs[5].clone(),
                it.regs[6].clone())))
        } else {
            Err(DauphinError::malformed("unexpected instruction"))
        }
    }
}

pub struct ExtractFilterCommand(Register,Register,Register,Register,Register,Register,Register);

impl Command for ExtractFilterCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![self.0.serialize(),self.1.serialize(),self.2.serialize(),
                     self.3.serialize(),self.4.serialize(),self.5.serialize(),
                     self.6.serialize()]))
    }
}

pub struct GapsCommandType();

impl CommandType for GapsCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 8,
            trigger: CommandTrigger::Command(std("gaps"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            Ok(Box::new(GapsCommand(
                it.regs[0].clone(),it.regs[1].clone(),it.regs[2].clone(),
                it.regs[3].clone(),it.regs[4].clone(),it.regs[5].clone(),
                it.regs[6].clone(),it.regs[7].clone())))
        } else {
            Err(DauphinError::malformed("unexpected instruction"))
        }
    }
}

pub struct GapsCommand(Register,Register,Register,Register,Register,Register,Register,Register);

impl Command for GapsCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![self.0.serialize(),self.1.serialize(),self.2.serialize(),
                     self.3.serialize(),self.4.serialize(),self.5.serialize(),
                     self.6.serialize(),self.7.serialize()]))
    }
}

pub struct RangeCommandType();

impl CommandType for RangeCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 4,
            trigger: CommandTrigger::Command(std("range"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            Ok(Box::new(RangeCommand(
                it.regs[0].clone(),it.regs[1].clone(),it.regs[2].clone(),
                it.regs[3].clone())))
        } else {
            Err(DauphinError::malformed("unexpected instruction"))
        }
    }
}

pub struct RangeCommand(Register,Register,Register,Register);

impl Command for RangeCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![self.0.serialize(),self.1.serialize(),self.2.serialize(),
                     self.3.serialize()]))
    }
}

pub struct SplitCharactersCommandType();

impl CommandType for SplitCharactersCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 4,
            trigger: CommandTrigger::Command(std("split_characters"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            Ok(Box::new(RangeCommand(
                it.regs[0].clone(),it.regs[1].clone(),it.regs[2].clone(),
                it.regs[3].clone())))
        } else {
            Err(DauphinError::malformed("unexpected instruction"))
        }
    }
}

pub struct SplitCharactersCommand(Register,Register,Register,Register);

impl Command for SplitCharactersCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![self.0.serialize(),self.1.serialize(),self.2.serialize(),
                     self.3.serialize()]))
    }
}

pub struct SetDifferenceCommandType();

impl CommandType for SetDifferenceCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 3,
            trigger: CommandTrigger::Command(std("set_difference"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            Ok(Box::new(SetDifferenceCommand(
                it.regs[0].clone(),it.regs[1].clone(),it.regs[2].clone())))
        } else {
            Err(DauphinError::malformed("unexpected instruction"))
        }
    }
}

pub struct SetDifferenceCommand(Register,Register,Register);

impl Command for SetDifferenceCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![self.0.serialize(),self.1.serialize(),self.2.serialize()]))
    }
}

pub struct AssertCommandType();

impl CommandType for AssertCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 2,
            trigger: CommandTrigger::Command(std("assert"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,_,_) = &it.itype {
            Ok(Box::new(AssertCommand(it.regs[0],it.regs[1])))
        } else {
            Err(DauphinError::malformed("unexpected instruction"))
        }
    }    
}

pub struct AssertCommand(Register,Register);

impl Command for AssertCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![self.0.serialize(),self.1.serialize()]))
    }

    fn simple_preimage(&self, context: &mut PreImageContext) -> anyhow::Result<PreImagePrepare> {
        Ok(if context.is_reg_valid(&self.0) && context.is_reg_valid(&self.1) && !context.is_last() {
            PreImagePrepare::Replace
        } else if let Some(a) = context.get_reg_size(&self.0) {
            PreImagePrepare::Keep(vec![(self.0.clone(),a)])
        } else {
            PreImagePrepare::Keep(vec![])
        })
    }
    
    fn preimage_post(&self, _context: &mut PreImageContext) -> anyhow::Result<PreImageOutcome> {
        Ok(PreImageOutcome::Replace(vec![]))
    }
}

pub struct HaltCommandType();

impl CommandType for HaltCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 1,
            trigger: CommandTrigger::Command(std("halt"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,_,_) = &it.itype {
            Ok(Box::new(HaltCommand(it.regs[0].clone())))
        } else {
            Err(DauphinError::malformed("unexpected instruction"))
        }
    }
}

pub struct HaltCommand(Register);

impl Command for HaltCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![self.0.serialize()]))
    }
}

pub struct AlienateCommandType();

impl CommandType for AlienateCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 0,
            trigger: CommandTrigger::Command(std("alienate"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,_,_) = &it.itype {
            Ok(Box::new(AlienateCommand(it.regs.clone())))
        } else {
            Err(DauphinError::malformed("unexpected instruction"))
        }
    }    
}

pub struct AlienateCommand(pub(crate) Vec<Register>);

impl Command for AlienateCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(None)
    }
    
    fn preimage(&self, context: &mut PreImageContext, _ic: Option<Box<dyn InterpCommand>>) -> anyhow::Result<PreImageOutcome> {
        for reg in self.0.iter() {
            context.set_reg_invalid(reg);
            context.set_reg_size(reg,None);
        }
        Ok(PreImageOutcome::Skip(vec![]))
    }
}

pub struct RulerIntervalCommandType();

impl CommandType for RulerIntervalCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 3,
            trigger: CommandTrigger::Command(Identifier::new("std","ruler_interval"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,_sig,_) = &it.itype {
            Ok(Box::new(RulerIntervalCommand(it.regs[0],it.regs[1],it.regs[2])))
        } else {
            Err(DauphinError::malformed("unexpected instruction"))
        }
    }    
}

pub struct RulerIntervalCommand(Register,Register,Register);

impl Command for RulerIntervalCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![self.0.serialize(),self.1.serialize(),self.2.serialize()]))
    }    
}

pub struct RulerMarkingsCommandType();

impl CommandType for RulerMarkingsCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 4,
            trigger: CommandTrigger::Command(Identifier::new("std","ruler_markings"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,_sig,_) = &it.itype {
            Ok(Box::new(RulerMarkingsCommand(it.regs[0],it.regs[1],it.regs[2],it.regs[3])))
        } else {
            Err(DauphinError::malformed("unexpected instruction"))
        }
    }    
}

pub struct RulerMarkingsCommand(Register,Register,Register,Register);

impl Command for RulerMarkingsCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![self.0.serialize(),self.1.serialize(),self.2.serialize(),self.3.serialize()]))
    }    
}

pub fn make_std() -> CompLibRegister {
    /* next is 38 */
    let mut set = CompLibRegister::new(&std_id(),Some(make_std_interp()));
    library_eq_command(&mut set);
    set.push("len",None,LenCommandType());
    set.push("assert",Some(4),AssertCommandType());
    set.push("alienate",Some(13),AlienateCommandType());
    set.push("halt",Some(30),HaltCommandType());
    set.push("print",Some(14),PrintCommandType());
    set.push("format",Some(2),FormatCommandType());
    set.push("bytes_to_bool",Some(25),BytesToBoolCommandType());
    set.push("derun",Some(26),DerunCommandType());
    set.push("run",Some(29),RunCommandType());
    set.push("nth",Some(27),NthCommandType());
    set.push("halt",Some(30),HaltCommandType());
    set.push("ruler_interval",Some(31),RulerIntervalCommandType());
    set.push("ruler_markings",Some(32),RulerMarkingsCommandType());
    set.push("comma_format",Some(33),CommaFormatCommandType());
    set.push("set_difference",Some(34),SetDifferenceCommandType());
    set.push("gaps",Some(35),GapsCommandType());
    set.push("range",Some(36),RangeCommandType());
    set.push("split_characters",Some(37),SplitCharactersCommandType());
    set.add_header("std",include_str!("header.dp"));
    library_numops_commands(&mut set);
    library_assign_commands(&mut set);
    library_vector_commands(&mut set);
    library_map_commands(&mut set);
    set.dynamic_data(include_bytes!("std-0.0.ddd"));
    set
}
