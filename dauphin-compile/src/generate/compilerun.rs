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
use super::gencontext::GenContext;
use crate::resolver::Resolver;
use crate::cli::Config;
use crate::command::{ CompilerLink, Instruction, InstructionType, PreImageOutcome };
use crate::model::{ PreImageContext };
use crate::util::DFloat;
use dauphin_interp::runtime::{ Register, InterpValue, InterpContext, lossless_numbers_to_indexes };
use dauphin_interp::util::{ DauphinError, error_locate_cb };

struct CompileRun<'a,'b,'d> {
    context: PreImageContext<'a,'b>,
    gen_context: &'a mut GenContext<'d>
}

impl<'a,'b,'d> CompileRun<'a,'b,'d> {
    pub fn new(context: &'b mut InterpContext, compiler_link: &CompilerLink, resolver: &'a Resolver, gen_context: &'a mut GenContext<'d>, 
                config: &Config, first: bool, last: bool) -> anyhow::Result<CompileRun<'a,'b,'d>> {
        let picontext = PreImageContext::new(context,compiler_link,Box::new(resolver),config,&gen_context.state().regalloc(),first,last);
        Ok(CompileRun {
            context: picontext,
            gen_context
        })
    }

    fn commit(&mut self) -> anyhow::Result<()> {
        let regs = self.context.context_mut().registers_mut().commit();
        for reg in &regs {
            if self.context.is_reg_valid(reg) {
                let len = self.context.context_mut().registers_mut().get(reg).borrow().get_shared()?.len();
                self.context.set_reg_size(reg,Some(len));
            }
        }
        Ok(())
    }

    fn unable_instr(&mut self, instr: &Instruction, sizes: &[(Register,usize)]) -> anyhow::Result<()> {
        self.add(instr.clone())?;
        self.commit()?;
        let changing = instr.itype.out_registers();
        for idx in &changing {
            self.context.set_reg_invalid(&instr.regs[*idx]);
            self.context.set_reg_size(&instr.regs[*idx],None);
        }
        for (reg,size) in sizes {
            self.context.set_reg_size(reg,Some(*size));
        }
        Ok(())
    }

    fn long_constant<F,T>(&mut self, reg: &Register, values: &Vec<T>, mut cb: F) -> anyhow::Result<()> where F: FnMut(Register,&T) -> Instruction {
        if values.len() == 1 {
            self.add(cb(*reg,&values[0]))?;
        } else {
            self.add(Instruction::new(InstructionType::Nil,vec![*reg]))?;
            for v in values {
                let inter = self.gen_context.state_mut().regalloc().allocate();
                self.add(cb(inter,v))?;
                self.add(Instruction::new(InstructionType::Append,vec![*reg,inter]))?;
            }
        }
        Ok(())
    }

    fn add(&mut self, instr: Instruction) -> anyhow::Result<()> {
        let command = self.context.linker().instruction_to_command(&instr)?.1;
        let time = command.execution_time(&self.context);
        self.gen_context.add_timed(instr,time);
        Ok(())
    }

    fn make_constant(&mut self, reg: &Register) -> anyhow::Result<()> {
        // XXX don't copy the big ones
        let value = self.context.context_mut().registers_mut().get(reg).borrow().get_shared()?;
        match value.as_ref() {
            InterpValue::Empty => {
                self.add(Instruction::new(InstructionType::Nil,vec![*reg]))?;
            },
            InterpValue::Indexes(indexes) => {
                self.add(Instruction::new(InstructionType::Const(indexes.to_vec()),vec![*reg]))?;
            },
            InterpValue::Numbers(numbers) => {
                if let Some(indexes) = lossless_numbers_to_indexes(numbers).ok() {
                    self.add(Instruction::new(InstructionType::Const(indexes.to_vec()),vec![*reg]))?;
                } else {
                    self.long_constant(reg,numbers,|r,n| {
                        Instruction::new(InstructionType::NumberConst(DFloat::new(*n)),vec![r])
                    })?;
                }
            },
            InterpValue::Boolean(bools) => {
                self.long_constant(reg,bools,|r,n| {
                    Instruction::new(InstructionType::BooleanConst(*n),vec![r])
                })?;
            },
            InterpValue::Strings(strings) => {
                self.long_constant(reg,strings,|r,n| {
                    Instruction::new(InstructionType::StringConst(n.clone()),vec![r])
                })?;
            },
            InterpValue::Bytes(bytes) => {
                self.long_constant(reg,bytes,|r,n| {
                    Instruction::new(InstructionType::BytesConst(n.clone()),vec![r])
                })?;
            },
        }
        Ok(())
    }

    fn preimage_instr(&mut self, instr: &Instruction) -> anyhow::Result<()> {
        //print!("{:?}",instr);
        let command = self.context.linker().instruction_to_command(instr)?.1;
        let ic = self.context.linker().instruction_to_interp_command(instr)?;
        match command.preimage(&mut self.context,ic)? {
            PreImageOutcome::Skip(sizes) => {
                self.unable_instr(&instr,&sizes)?;
            },
            PreImageOutcome::Replace(instrs) => {
                if self.context.is_last() {
                    Err(DauphinError::internal(file!(),line!()))?  /* Illegal replace during last run */
                }
                for instr in instrs {
                    self.preimage_instr(&instr)?;
                }                    
            },
            PreImageOutcome::Constant(regs) => {
                for reg in &regs {
                    self.context.set_reg_valid(reg)?;
                }
                self.commit()?;
                for reg in &regs {
                    self.make_constant(reg)?;
                }
            },
            PreImageOutcome::SkipConstant(regs) => {
                for reg in &regs {
                    self.context.set_reg_valid(reg)?;
                }
                self.commit()?;
                self.add(instr.clone())?;
            }
        }
        self.commit()?;
        Ok(())
    }

    pub fn preimage(&mut self) -> anyhow::Result<()> {
        for instr in &self.gen_context.get_instructions() {
            let out = self.preimage_instr(instr);
            error_locate_cb(|| {
                let line = self.context.context().get_line_number();
                (line.0.to_string(),line.1,0)
            },out).with_context(|| format!("preimaging {:?}",instr))?;
        }
        self.context.finish();
        self.gen_context.phase_finished();
        Ok(())
    }
}

pub fn compile_run(icontext: &mut InterpContext, compiler_link: &CompilerLink, resolver: &Resolver, context: &mut GenContext, config: &Config, first: bool, last: bool) -> anyhow::Result<()> {
    let mut pic = CompileRun::new(icontext,compiler_link,resolver,context,config,first,last)?;
    pic.preimage()?;
    Ok(())
}
