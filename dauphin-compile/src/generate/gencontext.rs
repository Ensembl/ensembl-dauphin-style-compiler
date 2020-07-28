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
use std::fmt;
use std::mem::swap;
use crate::command::{ Instruction, InstructionType };
use crate::generate::GenerateState;
use crate::model::{ DefStore, RegisterAllocator };
use crate::typeinf::{ ExpressionType, MemberType, TypeModel, Typing, get_constraint };
use dauphin_interp::runtime::Register;

pub struct GenContext<'a,'b> {
    input_instrs: Vec<(Instruction,f64)>,
    output_instrs: Vec<(Instruction,f64)>,
    generate_state: &'b mut GenerateState<'a>
}

impl<'a,'b> fmt::Debug for GenContext<'a,'b> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let instr_str : Vec<String> = self.input_instrs.iter().map(|v| format!("{:?}",v.0)).collect();
        write!(f,"{}\n",instr_str.join(""))?;
        Ok(())
    }
}

impl<'a,'b> GenContext<'a,'b> {
    pub fn new(generate_state: &'b mut GenerateState<'a>) -> GenContext<'a,'b> {
        GenContext {
            input_instrs: Vec::new(),
            output_instrs: Vec::new(),
            generate_state
        }
    }

    pub fn state(&self) -> &GenerateState<'a> { &self.generate_state }
    pub fn state_mut(&mut self) -> &mut GenerateState<'a> { &mut self.generate_state }

    pub fn get_instructions(&self) -> Vec<Instruction> {
        self.input_instrs.iter().map(|x| x.0.clone()).collect()
    }

    pub fn get_timed_instructions(&self) -> Vec<(Instruction,f64)> {
        self.input_instrs.to_vec()
    }

    pub fn add(&mut self, instr: Instruction) {
        self.output_instrs.push((instr,0.));
    }

    pub fn add_timed(&mut self, instr: Instruction, time: f64) {
        self.output_instrs.push((instr,time));
    }

    pub fn phase_finished(&mut self) {
        swap(&mut self.input_instrs, &mut self.output_instrs);
        self.output_instrs = Vec::new();
    }
}
