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
use crate::util::DauphinError;
use std::collections::HashMap;
use std::rc::Rc;
use crate::command::{ CommandInterpretSuite, InterpCommand };
use serde_cbor::Value as CborValue;
use crate::util::cbor::{ cbor_int, cbor_map, cbor_array, cbor_entry, cbor_string, cbor_map_iter };
use crate::runtime::{ InterpContext, PayloadFactory, Register };

pub(super) const VERSION : u32 = 0;

struct ProgramCursor<'a> {
    value: &'a Vec<CborValue>,
    index: usize
}

impl<'a> ProgramCursor<'a> {
    fn more(&self) -> bool {
        self.index < self.value.len()
    }

    fn next(&mut self) -> anyhow::Result<&'a CborValue> {
        if self.more() {
            self.index += 1;
            Ok(&self.value[self.index-1])
        } else {
            Err(DauphinError::malformed("premature termination of program"))
        }
    }

    fn next_n(&mut self, n: usize) -> anyhow::Result<Vec<&'a CborValue>> {
        (0..n).map(|_| self.next()).collect()
    }
}

pub struct InterpreterLinkProgram {
    commands: Vec<Box<dyn InterpCommand>>,
    instructions: Option<Vec<(String,Vec<Register>)>>
}

pub struct InterpreterLink {
    programs: HashMap<String,InterpreterLinkProgram>,
    payloads: HashMap<(String,String),Rc<Box<dyn PayloadFactory>>>
}

impl InterpreterLink {
    fn make_commands(ips: &CommandInterpretSuite, program: &CborValue) -> anyhow::Result<Vec<Box<dyn InterpCommand>>> {
        let mut cursor = ProgramCursor {
            value: cbor_array(program,0,true)?,
            index: 0
        };
        let mut out = vec![];
        while cursor.more() {
            let opcode = cbor_int(cursor.next()?,None)? as u32;
            let ds = ips.get_deserializer(opcode)?;
            let (_,num_args) = ds.get_opcode_len()?.ok_or_else(|| DauphinError::malformed("attempt to deserialize an unserializable"))?;
            let args = cursor.next_n(num_args)?;
            out.push(ds.deserialize(opcode,&args).context("deserializing program")?);
        }
        Ok(out)
    }

    pub fn add_payload<P>(&mut self, set: &str, name: &str, pf: P) where P: PayloadFactory + 'static {
        self.payloads.insert((set.to_string(),name.to_string()),Rc::new(Box::new(pf)));
    }

    fn make_instruction(cbor: &CborValue) -> anyhow::Result<(String,Vec<Register>)> {
        let data = cbor_array(cbor,2,false)?;
        let regs = cbor_array(&data[1],0,true)?.iter().map(|x| Register::deserialize(x)).collect::<Result<_,_>>()?;
        Ok((cbor_string(&data[0])?,regs))
    }

    fn make_instructions(cbor: &CborValue) -> anyhow::Result<Vec<(String,Vec<Register>)>> {
        cbor_array(cbor,0,true)?.iter().map(|x| InterpreterLink::make_instruction(x)).collect()
    }

    fn get_program<'a>(&'a self, name: &str) -> anyhow::Result<&'a InterpreterLinkProgram> {
        Ok(self.programs.get(name).ok_or_else(|| DauphinError::config(&format!("No such program {}",name)))?)
    }

    fn new_real(mut ips: CommandInterpretSuite, cbor: &CborValue) -> anyhow::Result<InterpreterLink> {
        let mut out = InterpreterLink {
            programs: HashMap::new(),
            payloads: ips.copy_payloads()
        };
        let data = cbor_map(cbor,&vec!["version","suite","programs"])?;
        ips.adjust(data[1])?;
        let got_ver = cbor_int(data[0],None)? as u32;
        if got_ver != VERSION {
            return Err(DauphinError::integration(&format!("Incompatible code. got v{} understand v{}",got_ver,VERSION)));
        }
        for (name,program) in cbor_map_iter(data[2])? {
            let name = cbor_string(name)?;
            let cmds = cbor_entry(program,"cmds")?.ok_or_else(|| DauphinError::malformed("missing cmds section"))?;
            let symbols = cbor_entry(program,"symbols")?;
            out.programs.insert(name.to_string(),InterpreterLinkProgram {
                commands: InterpreterLink::make_commands(&ips,cmds).context("building commands")?,
                instructions: symbols.map(|x| InterpreterLink::make_instructions(x)).transpose().context("building debug symbols")?
            });
        }
        Ok(out)
    }

    pub fn new(mut ips: CommandInterpretSuite, cbor: &CborValue) -> anyhow::Result<InterpreterLink> {
        InterpreterLink::new_real(ips,cbor).context("parsing program")
    }

    pub fn get_commands(&self, name: &str) -> anyhow::Result<&Vec<Box<dyn InterpCommand>>> {
        Ok(&self.get_program(name)?.commands)
    }

    pub fn get_instructions(&self, name: &str) -> anyhow::Result<Option<&Vec<(String,Vec<Register>)>>> { 
        Ok(self.get_program(name)?.instructions.as_ref())
    }

    pub(crate) fn new_context(&self) -> InterpContext {
        InterpContext::new(&self.payloads)
    }
}
