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
use std::pin::Pin;
use std::future::Future;
use std::rc::Rc;
use std::slice::Iter;
use crate::command::{ InterpCommand, InterpreterLink, CommandResult };
use crate::util::{ DauphinError, error_locate_cb };
use crate::runtime::{ Register, InterpContext };

pub trait InterpretInstance {
    fn more<'a>(&'a mut self) -> Pin<Box<dyn Future<Output=anyhow::Result<bool>> + 'a>>;
}

struct CommandGetter {
    commands: Rc<Vec<Box<dyn InterpCommand>>>,
    index: usize
}

impl CommandGetter {
    pub fn new(commands: Rc<Vec<Box<dyn InterpCommand>>>) -> CommandGetter {
        CommandGetter {
            commands,
            index: 0
        }
    }

    pub fn next(&mut self) -> Option<&Box<dyn InterpCommand>> {
        if self.index < self.commands.len() {
            self.index += 1;
            Some(&self.commands[self.index-1])
        } else {
            None
        }
    }

    pub fn index(&self) -> usize { self.index }
}

async fn more_internal(commands: &mut CommandGetter, context: &mut InterpContext) -> anyhow::Result<bool> {
    while let Some(command) = commands.next() {
        match command.execute(context)? {
            CommandResult::SyncResult() => {},
            CommandResult::AsyncResult(asy) => {
                asy.execute(context).await?;
            }
        }
        context.registers_mut().commit();
        if context.test_pause() {
            return Ok(true);
        }
    }
    Ok(false)
}

async fn more(commands: &mut CommandGetter, context: &mut InterpContext) -> anyhow::Result<bool> {
    let out = more_internal(commands,context).await;
    error_locate_cb(|| {
        let line = context.get_line_number();
        (line.0.to_string(),line.1,commands.index())
    },out)
}

pub struct PartialInterpretInstance<'b> {
    commands: CommandGetter,
    context: &'b mut InterpContext
}

impl<'b> PartialInterpretInstance<'b> {
    pub fn new(interpret_linker: &InterpreterLink, name: &str, context: &'b mut InterpContext) -> anyhow::Result<PartialInterpretInstance<'b>> {
        Ok(PartialInterpretInstance {
            commands: CommandGetter::new(interpret_linker.get_commands(name)?),
            context
        })
    }
}

impl<'b> InterpretInstance for PartialInterpretInstance<'b> {
    fn more<'c>(&'c mut self) -> Pin<Box<dyn Future<Output=anyhow::Result<bool>> + 'c>> {
        Box::pin(more(&mut self.commands, &mut self.context))
    }
}

pub struct StandardInterpretInstance {
    commands: CommandGetter,
    context: InterpContext
}

impl StandardInterpretInstance {
    pub fn new(interpret_linker: &InterpreterLink, name: &str) -> anyhow::Result<StandardInterpretInstance> {
        Ok(StandardInterpretInstance {
            commands: CommandGetter::new(interpret_linker.get_commands(name)?),
            context: InterpContext::new()
        })
    }
    
    pub fn context_mut(&mut self) -> &mut InterpContext { &mut self.context }
}

impl InterpretInstance for StandardInterpretInstance {
    fn more<'b>(&'b mut self) -> Pin<Box<dyn Future<Output=anyhow::Result<bool>> + 'b>> {
        Box::pin(more(&mut self.commands, &mut self.context))
    }
}

pub struct DebugInterpretInstance<'b> {
    commands: CommandGetter,
    context: &'b mut InterpContext,
    instrs: Vec<(String,Vec<Register>)>,
    index: usize
}

impl<'b> DebugInterpretInstance<'b> {
    pub fn new(interpret_linker: &InterpreterLink, instrs: &[(String,Vec<Register>)], name: &str, context: &'b mut InterpContext) -> anyhow::Result<DebugInterpretInstance<'b>> {
        Ok(DebugInterpretInstance {
            commands: CommandGetter::new(interpret_linker.get_commands(name)?),
            context,
            instrs: instrs.to_vec(),
            index: 0
        })
    }

    async fn more_internal(&mut self) -> anyhow::Result<bool> {
        let idx = self.index;
        self.index += 1;
        if let Some(command) = self.commands.next() {
            let (instr,regs) = &self.instrs[idx];
            print!("{}",self.context.registers_mut().dump_many(&regs)?);
            print!("{}",instr);
            match command.execute(&mut self.context)? {
                CommandResult::SyncResult() => {},
                CommandResult::AsyncResult(asy) => {
                    asy.execute(&mut self.context).await?;
                }
            }
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

impl<'b> InterpretInstance for DebugInterpretInstance<'b> {
    fn more<'c>(&'c mut self) -> Pin<Box<dyn Future<Output=anyhow::Result<bool>> + 'c>> {
        Box::pin(async move {
            let out = self.more_internal().await;
            error_locate_cb(|| {
                let line = self.context.get_line_number();
                (line.0.to_string(),line.1,self.commands.index())
            },out)
        })
    }
}
