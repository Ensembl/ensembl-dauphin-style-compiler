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
use std::slice::Iter;
use crate::command::{ InterpCommand, InterpreterLink };
use crate::util::{ DauphinError, error_locate_cb };
use crate::runtime::{ Register, InterpContext };

pub trait InterpretInstance {
    fn more(&mut self) -> anyhow::Result<bool>;
}

pub struct StandardInterpretInstance<'a,'b> {
    commands: Iter<'a,Box<dyn InterpCommand>>,
    context: &'b mut InterpContext
}

impl<'a,'b> StandardInterpretInstance<'a,'b> {
    pub fn new(interpret_linker: &'a InterpreterLink, name: &str, context: &'b mut InterpContext) -> anyhow::Result<StandardInterpretInstance<'a,'b>> {
        Ok(StandardInterpretInstance {
            commands: interpret_linker.get_commands(name)?.iter(),
            context
        })
    }

    fn more_internal(&mut self) -> anyhow::Result<bool> {
        while let Some(command) = self.commands.next() {
            command.execute(self.context)?;
            self.context.registers_mut().commit();
            if self.context.test_pause() {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl<'a,'b> InterpretInstance for StandardInterpretInstance<'a,'b> {
    fn more(&mut self) -> anyhow::Result<bool> {
        let out = self.more_internal();
        error_locate_cb(|| {
            let line = self.context.get_line_number();
            (line.0.to_string(),line.1)
        },out)
    }
}

pub struct DebugInterpretInstance<'a,'b> {
    commands: Iter<'a,Box<dyn InterpCommand>>,
    context: &'b mut InterpContext,
    instrs: Vec<(String,Vec<Register>)>,
    index: usize
}

impl<'a,'b> DebugInterpretInstance<'a,'b> {
    pub fn new(interpret_linker: &'a InterpreterLink, instrs: &[(String,Vec<Register>)], name: &str, context: &'b mut InterpContext) -> anyhow::Result<DebugInterpretInstance<'a,'b>> {
        Ok(DebugInterpretInstance {
            commands: interpret_linker.get_commands(name)?.iter(),
            context,
            instrs: instrs.to_vec(),
            index: 0
        })
    }

    fn more_internal(&mut self) -> anyhow::Result<bool> {
        let idx = self.index;
        self.index += 1;
        if let Some(command) = self.commands.next() {
            let (instr,regs) = &self.instrs[idx];
            print!("{}",self.context.registers_mut().dump_many(&regs)?);
            print!("{}",instr);
            command.execute(self.context)?;
            self.context.registers_mut().commit();
            print!("{}",self.context.registers_mut().dump_many(&regs)?);
            if self.context.test_pause() {
                print!("PAUSE\n");
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a,'b> InterpretInstance for DebugInterpretInstance<'a,'b> {
    fn more(&mut self) -> anyhow::Result<bool> {
        let out = self.more_internal();
        error_locate_cb(|| {
            let line = self.context.get_line_number();
            (line.0.to_string(),line.1)
        },out)
    }
}
