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
use std::rc::Rc;
use crate::runtime::{ Register, InterpContext, InterpValue };

pub trait VectorSource {
    fn len(&self, context: &mut InterpContext, index: usize) -> anyhow::Result<usize>;
    fn get_shared(&self, context: &mut InterpContext, index: usize) -> anyhow::Result<Rc<InterpValue>>;
    fn get_exclusive(&self, context: &mut InterpContext, index: usize) -> anyhow::Result<InterpValue>;
    fn set(&self, context: &mut InterpContext, index: usize, value: InterpValue);
}

pub struct RegisterVectorSource<'c> {
    regs: &'c [Register]
}

impl<'c> RegisterVectorSource<'c> {
    pub fn new(regs: &'c [Register]) -> RegisterVectorSource<'c> {
        RegisterVectorSource {
            regs
        }
    }
}

impl<'c> VectorSource for RegisterVectorSource<'c> {
    fn len(&self, context: &mut InterpContext, index: usize) -> anyhow::Result<usize> {
        context.registers_mut().len(&self.regs[index])
    }

    fn get_shared(&self, context: &mut InterpContext, index: usize) -> anyhow::Result<Rc<InterpValue>> {
        let r = context.registers_mut().get(&self.regs[index]);
        let r = r.borrow();
        r.get_shared()
    }

    fn get_exclusive(&self, context: &mut InterpContext, index: usize) -> anyhow::Result<InterpValue> {
        let r = context.registers_mut().get(&self.regs[index]);
        let mut r = r.borrow_mut();
        r.get_exclusive()
    }

    fn set(&self, context: &mut InterpContext, index: usize, value: InterpValue) {
        context.registers_mut().write(&self.regs[index],value);
    }
}

impl<'c> RegisterVectorSource<'c> {
    pub fn copy(&self, context: &mut InterpContext, dst: usize, src: usize) -> anyhow::Result<()> {
        context.registers_mut().copy(&self.regs[dst],&self.regs[src])?;
        Ok(())
    }
}
