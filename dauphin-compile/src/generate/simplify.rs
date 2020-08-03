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
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::command::{ Instruction, InstructionType };
use crate::util::DFloat;
use crate::model::{ StructDef, EnumDef };
use crate::typeinf::{ ContainerType, MemberType };
use crate::generate::GenerateState;
use super::gencontext::GenContext;
use dauphin_interp::command::Identifier;
use dauphin_interp::runtime::Register;
use dauphin_interp::types::BaseType;
use dauphin_interp::util::DauphinError;

/* simplification is the process of converting arbitrary assemblies of structs, enums and vecs into sets of vecs of
 * simple values. To achieve this, vecs of structured types are converted to sets of vecs of simpler types.
 * 
 * dauphin datastructures cannot be defined recursively, so they can be ordered such that within the ordering
 * containment occurs in only one direction. With such an order and starting at the largest type, data structures
 * are simplified iteratively. After the complete elimination of one structure to generate new code, the code is
 * considered completely afresh to eliminate the next.
 * 
 * For each elimination, first registers are made and then each instruction updated in turn.
 * 
 * Registers are made in two stages. First they are allocated and then refs are updated. refs refer to some origin
 * register of the same type. Because registers have just been allocated for the whole type, there are now matching
 * sets of registers to replace the ref and non-ref types. For each reference, the origin is updated to point to the
 * relevant sub-register, copying the path. Any of those reference registers which refer to the type currently being 
 * split are then replaced with a new origin and this part removed from the expression.
 * 
 * Extension proceeds slightly differently depending on whether a struct or an enum is extended. However, most
 * instructions are extended in the same way and are handled in a common function.
 */

fn number_reg(state: &mut GenerateState) -> Register {
    let out = state.regalloc().allocate();
    state.types_mut().set(&out,&MemberType::Base(BaseType::NumberType));
    out
}

macro_rules! instr {
    ($context:expr,$type:ident,$($regs:expr),*) => {
        $context.add(Instruction::new(InstructionType::$type,vec![$($regs),*]));
    };
}

macro_rules! instr_f {
    ($context:expr,$type:ident,$itype:ident,$($regs:expr),*) => {
        {
            let state = $context.state_mut();
            let x = state.regalloc().allocate();
            state.types_mut().set(&x,&MemberType::Base(BaseType::$type));
            instr!($context,$itype,x,$($regs),*);
            x
        }
    };
}

fn allocate_registers(state: &mut GenerateState, member_types: &Vec<MemberType>, with_index: bool, container_type: ContainerType) -> Vec<Register> {
    let mut out = Vec::new();
    if with_index {
        let reg = number_reg(state);
        state.types_mut().set(&reg,&container_type.construct(MemberType::Base(BaseType::NumberType)));
        out.push(reg);
    }
    for member_type in member_types.iter() {
        let reg = state.regalloc().allocate();
        state.types_mut().set(&reg,&container_type.construct(member_type.clone()));
        out.push(reg);
    }
    out
}

fn extend_vertical<F>(in_: &Vec<Register>, mapping: &SimplifyTypeMapperResult,mut cb: F) -> anyhow::Result<()>
        where F: FnMut(Vec<Register>) -> anyhow::Result<()> {
    let mut expanded = Vec::new();
    let mut len = None;
    for in_reg in in_.iter() {
        let map = mapping.get(&in_reg).unwrap_or(Rc::new(vec![*in_reg])).clone();
        if len.is_none() { len = Some(map.len()); }
        if map.len() != len.unwrap() { return Err(DauphinError::internal(file!(),line!())); /* mismatched register lengths */ }
        expanded.push(map);
    }
    for i in 0..len.unwrap() {
        let here_regs : Vec<Register> = expanded.iter().map(|v| v[i].clone()).collect();
        cb(here_regs)?;
    }
    Ok(())
}

/* Some easy value for unused enum branches */
fn build_nil(context: &mut GenContext, reg: &Register, type_: &MemberType) -> anyhow::Result<()> {
    match type_ {
        MemberType::Vec(m) =>  {
            let state = context.state_mut();
            let subreg = state.regalloc().allocate();
            state.types_mut().set(&subreg,m);
            instr!(context,Nil,subreg);
            instr!(context,Star,*reg,subreg);
        },
        MemberType::Base(b) => match b {
            BaseType::BooleanType => context.add(Instruction::new(InstructionType::BooleanConst(false),vec![*reg])),
            BaseType::StringType => context.add(Instruction::new(InstructionType::StringConst(String::new()),vec![*reg])),
            BaseType::NumberType => context.add(Instruction::new(InstructionType::NumberConst(DFloat::new_usize(0)),vec![*reg])),
            BaseType::BytesType => context.add(Instruction::new(InstructionType::BytesConst(vec![]),vec![*reg])),
            BaseType::Invalid =>  return Err(DauphinError::internal(file!(),line!())),
            BaseType::StructType(name) => {
                let decl = context.state().defstore().get_struct_id(name)?;
                let mut subregs = vec![*reg];
                for member_type in decl.get_member_types() {
                    let state = context.state_mut();
                    let r = state.regalloc().allocate();
                    state.types_mut().set(&r,member_type);
                    build_nil(context,&r,member_type)?;
                    subregs.push(r);
                }
                context.add(Instruction::new(InstructionType::CtorStruct(name.clone()),subregs));
            },
            BaseType::EnumType(name) => {
                let state = context.state_mut();
                let decl = state.defstore().get_enum_id(name)?;
                let branch_type = decl.get_branch_types().get(0).ok_or_else(|| DauphinError::internal(file!(),line!()))?;
                let field_name = decl.get_names().get(0).ok_or_else(|| DauphinError::internal(file!(),line!()))?;
                let subreg = state.regalloc().allocate();
                state.types_mut().set(&subreg,branch_type);
                build_nil(context,&subreg,branch_type)?;
                context.add(Instruction::new(InstructionType::CtorEnum(name.clone(),field_name.clone()),vec![*reg,subreg]));
            }
        }
    }
    Ok(())
}

fn extend_common(context: &mut GenContext, instr: &Instruction, mapping: &SimplifyTypeMapperResult) -> anyhow::Result<()> {
    Ok(match &instr.itype {
        InstructionType::Proc(_,_) |
        InstructionType::Operator(_) |
        InstructionType::Run |
        InstructionType::Length |
        InstructionType::Add |
        InstructionType::SeqFilter |
        InstructionType::Pause(_) |
        InstructionType::SeqAt |
        InstructionType::NilValue(_) =>
            panic!("Impossible instruction! {:?}",instr),

        InstructionType::CtorStruct(_) |
        InstructionType::CtorEnum(_,_) |
        InstructionType::SValue(_,_) |
        InstructionType::RefSValue(_,_) |
        InstructionType::EValue(_,_) |
        InstructionType::RefEValue(_,_) |
        InstructionType::FilterEValue(_,_) |
        InstructionType::ETest(_,_) |
        InstructionType::NumEq |
        InstructionType::ReFilter |
        InstructionType::Const(_) |
        InstructionType::NumberConst(_) |
        InstructionType::BooleanConst(_) |
        InstructionType::StringConst(_) |
        InstructionType::BytesConst(_) |
        InstructionType::LineNumber(_) =>
            context.add(instr.clone()),

        InstructionType::Nil |
        InstructionType::Alias |
        InstructionType::Copy |
        InstructionType::Append |
        InstructionType::Square |
        InstructionType::RefSquare |
        InstructionType::Star => {
            extend_vertical(&instr.regs,mapping,|regs| {
                context.add(Instruction::new(instr.itype.clone(),regs));
                Ok(())
            })?
        },

        InstructionType::FilterSquare => {
            if let Some(srcs) = mapping.get(&instr.regs[1]) {
                instr!(context,FilterSquare,instr.regs[0],srcs[0]);
            } else {
                instr!(context,FilterSquare,instr.regs[0],instr.regs[1]);
            }
        },

        InstructionType::At => {
            if let Some(srcs) = mapping.get(&instr.regs[1]) {
                instr!(context,At,instr.regs[0],srcs[0]);
            } else {
                context.add(instr.clone());
            }
        },

        InstructionType::Filter => {
            extend_vertical(&vec![instr.regs[0],instr.regs[1]],mapping,|r| {
                instr!(context,Filter,r[0],r[1],instr.regs[2]);
                Ok(())
            })?
        },
        InstructionType::Call(name,impure,type_,flow) => {
            let mut new_regs = Vec::new();
            for reg in &instr.regs {
                if let Some(dests) = mapping.get(&reg) {
                    new_regs.extend(dests.iter().cloned());
                } else {
                    new_regs.push(reg.clone());
                }
            }
            context.add(Instruction::new(InstructionType::Call(name.clone(),*impure,type_.clone(),flow.clone()),new_regs));
        }
    })
}

fn extend_struct_instr(obj_name: &Identifier, context: &mut GenContext, decl: &StructDef, instr: &Instruction, mapping: &SimplifyTypeMapperResult) -> anyhow::Result<()> {
    /* because types topologically ordered and non-recursive
    * we know there's nothing to expand in the args in the else branches.
    */
    Ok(match &instr.itype {
        InstructionType::CtorStruct(name) => {
            if name == obj_name {
                let dests = mapping.get_or_fail(&instr.regs[0])?;
                for i in 1..instr.regs.len() {
                    instr!(context,Copy,dests[i-1],instr.regs[i]);
                }
            } else {
                context.add(instr.clone());
            }
        },

        InstructionType::SValue(name,field) if name == obj_name => {
            let dests = mapping.get_or_fail(&instr.regs[1])?;
            let pos = decl.get_names().iter().position(|n| n==field).ok_or_else(|| DauphinError::source(&format!("No such field {}\n",field)))?;
            instr!(context,Copy,instr.regs[0],dests[pos]);
        },

        InstructionType::RefSValue(name,field) if name == obj_name => {
            let dests = mapping.get_or_fail(&instr.regs[1])?;
            let pos = decl.get_names().iter().position(|n| n==field).ok_or_else(|| DauphinError::source(&format!("No such field {}\n",field)))?;
            instr!(context,Alias,instr.regs[0],dests[pos]);
        },

        _ => extend_common(context,instr,mapping)?
    })
}

fn extend_enum_instr(context: &mut GenContext, obj_name: &Identifier, decl: &EnumDef, instr: &Instruction, mapping: &SimplifyTypeMapperResult) -> anyhow::Result<()> {
    /* because types topologically ordered and non-recursive we know there's nothing to expand in the args */
    Ok(match &instr.itype {
        InstructionType::CtorEnum(name,field) => {
            if name == obj_name {
                let pos = decl.get_names().iter().position(|v| v==field).ok_or_else(|| DauphinError::source(&format!("No such field {}\n",field)))?;
                let dests = mapping.get_or_fail(&instr.regs[0])?;
                for i in 1..dests.len() {
                    if i-1 == pos {
                        context.add(Instruction::new(InstructionType::NumberConst(DFloat::new_usize(i-1)),vec![dests[0]]));
                        instr!(context,Copy,dests[i],instr.regs[1]);
                    } else {
                        let state = context.state_mut();
                        let type_ = state.types().get(&dests[i]).ok_or_else(|| DauphinError::internal(file!(),line!()))?.clone();
                        build_nil(context,&dests[i],&type_)?;
                    }
                }
            } else {
                context.add(instr.clone());
            }
        },

        InstructionType::FilterEValue(name,field) if name == obj_name => {
            let pos = decl.get_names().iter().position(|v| v==field).ok_or_else(|| DauphinError::source(&format!("No such field {}\n",field)))?;
            let srcs = mapping.get_or_fail(&instr.regs[1])?;
            let state = context.state_mut();
            let posreg = number_reg(state);
            context.add(Instruction::new(InstructionType::NumberConst(DFloat::new_usize(pos)),vec![posreg]));
            let seq = instr_f!(context,NumberType,At,srcs[0]);
            let filter = instr_f!(context,BooleanType,NumEq,srcs[0],posreg);
            instr!(context,Filter,instr.regs[0],seq,filter);
        },

        InstructionType::EValue(name,field) if name == obj_name => {
            let pos = decl.get_names().iter().position(|v| v==field).ok_or_else(|| DauphinError::source(&format!("No such field {}\n",field)))?;
            let srcs = mapping.get_or_fail(&instr.regs[1])?;
            let state = context.state_mut();
            let posreg = number_reg(state);
            context.add(Instruction::new(InstructionType::NumberConst(DFloat::new_usize(pos)),vec![posreg]));
            let filter = instr_f!(context,BooleanType,NumEq,srcs[0],posreg);
            instr!(context,Filter,instr.regs[0],srcs[pos+1],filter);
        },

        InstructionType::RefEValue(name,field) if name == obj_name => {
            let pos = decl.get_names().iter().position(|v| v==field).ok_or_else(|| DauphinError::source(&format!("No such field {}\n",field)))?;
            let srcs = mapping.get_or_fail(&instr.regs[1])?;
            instr!(context,Alias,instr.regs[0],srcs[pos+1]);
        },

        InstructionType::ETest(name,field) if name == obj_name => {
            let pos = decl.get_names().iter().position(|v| v==field).ok_or_else(|| DauphinError::source(&format!("No such field {}\n",field)))?;
            let srcs = mapping.get_or_fail(&instr.regs[1])?;
            let state = context.state_mut();
            let posreg = number_reg(state);
            context.add(Instruction::new(InstructionType::NumberConst(DFloat::new_usize(pos)),vec![posreg]));
            instr!(context,NumEq,instr.regs[0],srcs[0],posreg);
        },

        _ => extend_common(context,instr,mapping)?
    })
}

#[derive(Clone)]
struct SimplifyTypeMapperResult(HashMap<Register,Rc<Vec<Register>>>);

impl SimplifyTypeMapperResult {
    fn get(&self, reg: &Register) -> Option<Rc<Vec<Register>>> {
        self.0.get(reg).cloned()
    }

    fn get_or_fail(&self, reg: &Register) -> anyhow::Result<Rc<Vec<Register>>> {
        self.get(reg).ok_or_else(|| DauphinError::internal(file!(),line!()))
    }
}

struct SimplifyTypeMapper {
    member_types: Vec<MemberType>,
    with_index: bool,
    maps: SimplifyTypeMapperResult
}

impl SimplifyTypeMapper {
    fn new(state: &GenerateState, base: &BaseType) -> anyhow::Result<SimplifyTypeMapper> {
        let (member_types,with_index) = if let BaseType::EnumType(name) = base {
            let decl = state.defstore().get_enum_id(name)?;
            (decl.get_branch_types().to_vec(),true) 
        } else if let BaseType::StructType(name) = base {
            let decl = state.defstore().get_struct_id(name)?;
            (decl.get_member_types().to_vec(),false)
        } else {
            return Err(DauphinError::internal(file!(),line!()));
        };
        Ok(SimplifyTypeMapper {
            member_types,
            with_index,
            maps: SimplifyTypeMapperResult(HashMap::new())
        })
    }

    fn add(&mut self, state: &mut GenerateState, reg: &Register) -> anyhow::Result<()> {
        let type_ = state.types().get(reg).ok_or_else(|| DauphinError::internal(file!(),line!()))?.clone();
        let maps = &mut self.maps.0;
        if !maps.contains_key(reg) {
            maps.insert(reg.clone(),Rc::new(allocate_registers(state,&self.member_types,self.with_index,type_.get_container())));
        }
        Ok(())
    }

    fn get_result(&self) -> SimplifyTypeMapperResult {
        self.maps.clone()
    }
}

#[derive(Clone)]
pub struct SimplifyMapperData(Rc<RefCell<HashMap<BaseType,SimplifyTypeMapper>>>);

impl SimplifyMapperData {
    pub fn new() -> SimplifyMapperData {
        SimplifyMapperData(Rc::new(RefCell::new(HashMap::new())))
    }
}

struct SimplifyMapper {
    data: SimplifyMapperData
}

impl SimplifyMapper {
    fn new(data: &SimplifyMapperData) -> SimplifyMapper {
        SimplifyMapper {
            data: data.clone()
        }
    }

    fn get_all_registers(&self, state: &mut GenerateState, base: &BaseType) -> Vec<Register> {
        let mut target_registers = vec![];
        /* which registers will we be expanding? */
        for (reg,reg_type) in state.types().each_register() {
            if &reg_type.get_base() == base {
                target_registers.push(reg.clone());
            }
        }
        target_registers.sort();
        target_registers    
    }

    fn process_type(&mut self, state: &mut GenerateState, base: BaseType) -> anyhow::Result<SimplifyTypeMapperResult> {
        let mut types = self.data.0.borrow_mut();
        if !types.contains_key(&base) {
            types.insert(base.clone(),SimplifyTypeMapper::new(state,&base)?);
        }
        let regs = self.get_all_registers(state,&base);
        let stm = types.get_mut(&base).unwrap();
        for reg in &regs {
            stm.add(state,&reg)?;
        }
        Ok(types.get(&base).unwrap().get_result())
    }

    fn process_identifier(&mut self, state: &mut GenerateState, name: &Identifier) -> anyhow::Result<SimplifyTypeMapperResult> {
        Ok(if let Some(_) = state.defstore().get_struct_id(name).ok() {
            let base = BaseType::StructType(name.clone());
            self.process_type(state,base)?
        } else if let Some(_) = state.defstore().get_enum_id(name).ok() {
            let base = BaseType::EnumType(name.clone());
            self.process_type(state,base)?
        } else {
            return Err(DauphinError::internal(file!(),line!())); /* can only extend structs/enums */
        })
    }
}

fn extend_one(context: &mut GenContext, name: &Identifier) -> anyhow::Result<()> {
    let mut sm = SimplifyMapper::new(&context.state().simplify_mapper());
    let stm = sm.process_identifier(context.state_mut(),name)?;
    if let Some(decl) = context.state().defstore().get_struct_id(name).ok() {
        for instr in &context.get_instructions() {
            extend_struct_instr(name,context,&decl,instr,&stm)?;
        }
    } else if let Some(decl) = context.state().defstore().get_enum_id(name).ok() {
        for instr in &context.get_instructions() {
            extend_enum_instr(context,name,&decl,instr,&stm)?;
        }
    } else {
        return Err(DauphinError::internal(file!(),line!())); /* can only extend structs/enums */
    };
    context.phase_finished();
    Ok(())
}

fn remove_nils(context: &mut GenContext) -> anyhow::Result<()> {
    for instr in &context.get_instructions() {
        match &instr.itype {
            InstructionType::NilValue(typ) => {
                build_nil(context,&instr.regs[0],&typ)?;
            },
            _ => {
                context.add(instr.clone());
            }
        }
    }
    context.phase_finished();
    Ok(())
}

pub fn simplify(context: &mut GenContext) -> anyhow::Result<()> {
    remove_nils(context)?;
    let ids : Vec<Identifier> = context.state().defstore().get_structenum_order().cloned().collect::<Vec<_>>();
    for name in ids.iter().rev() {
        extend_one(context,&name)?;
    }
    Ok(())
}
