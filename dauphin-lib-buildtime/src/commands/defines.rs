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
use dauphin_interp::command::{ Identifier, InterpCommand };
use dauphin_interp::runtime::{ InterpValue, Register };
use dauphin_interp::util::DauphinError;
use dauphin_compile::command::{ Command, CommandSchema, CommandType, CommandTrigger, PreImageOutcome, Instruction };
use dauphin_compile::model::{ PreImageContext };
use serde_cbor::Value as CborValue;

pub struct DefineCommandType(pub bool);

impl CommandType for DefineCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 2,
            trigger: CommandTrigger::Command(Identifier::new("buildtime", if self.0 { "get_define" } else { "is_defined" }))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        Ok(Box::new(DefineCommand(self.0,it.regs[0],it.regs[1])))
    }    
}

pub struct DefineCommand(bool,Register,Register);

impl Command for DefineCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Err(DauphinError::internal(file!(),line!()))
    }

    fn preimage(&self, context: &mut PreImageContext, _ic: Option<Box<dyn InterpCommand>>) -> anyhow::Result<PreImageOutcome> {
        if context.is_reg_valid(&self.2) {
            let keys = context.context().registers().get_strings(&self.2)?;
            let config = context.config();
            let mut found = vec![];
            for key in keys.iter() {
                let mut value = None;
                for (k,v) in config.get_defines().iter() {
                    if k == key {
                        value = Some(v.to_string());
                    }
                }
                found.push(value);
            }
            if self.0 {
                let values : Vec<String> = found.drain(..).map(|v| v.unwrap_or_else(|| "".to_string())).collect();
                context.context_mut().registers_mut().write(&self.1,InterpValue::Strings(values));
            } else {
                let values : Vec<bool> = found.drain(..).map(|v| v.is_some()).collect();
                context.context_mut().registers_mut().write(&self.1,InterpValue::Boolean(values));
            }
            Ok(PreImageOutcome::Constant(vec![self.1]))
        } else {
            Err(DauphinError::runtime("buildtime::define needs key to be known at build time"))
        }
    }
}

#[cfg(test)]
mod test {
    use dauphin_test_harness::{ compile, xxx_test_config };
    use crate::test::{ make_compiler_suite, make_interpret_suite };

    #[test]
    fn defines_smoke() {
        let mut config = xxx_test_config();
        config.add_define(("yes".to_string(),"".to_string()));
        config.add_define(("hello".to_string(),"world".to_string()));
        let cs = make_compiler_suite(&config).expect("x");
        let is = make_interpret_suite().expect("y");
        let strings = compile(cs,&is,&config,"search:buildtime/defines").expect("a");
        for s in &strings {
            print!("{}\n",s);
        }
    }
}
