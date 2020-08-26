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
use dauphin_interp::util::{ DauphinError };
use futures::executor::block_on;
use std::time::{ SystemTime, Duration };
use std::collections::HashMap;
use std::rc::Rc;
use dauphin_compile::cli::Config;
use dauphin_compile::command::{ CommandCompileSuite, CompilerLink, Instruction, ProgramMetadataBuilder };
use dauphin_interp::command::{ CommandInterpretSuite, InterpreterLink };
use dauphin_interp::runtime::{ InterpContext, InterpValue, Register, PartialInterpretInstance, DebugInterpretInstance, InterpretInstance };
use dauphin_interp::stream::{ ConsoleStreamFactory, Stream };
use dauphin_interp::util::cbor::{ cbor_serialize };
use dauphin_compile::generate::{ generate, GenerateState };
use dauphin_compile::resolver::common_resolver;
use dauphin_compile::lexer::Lexer;
use dauphin_compile::parser::Parser;
use crate::hexdump;

pub fn interpreter<'a>(context: &'a mut InterpContext, interpret_linker: &'a InterpreterLink, config: &Config, name: &str) -> anyhow::Result<Box<dyn InterpretInstance + 'a>> {
    if let Some(instrs) = interpret_linker.get_instructions(name)? {
        if config.get_debug_run() {
            return Ok(Box::new(DebugInterpretInstance::new(interpret_linker,&instrs,name,context)?));
        }
    }
    Ok(Box::new(PartialInterpretInstance::new(interpret_linker,name,context)?))
}

fn export_indexes(ic: &mut InterpContext) -> anyhow::Result<HashMap<Register,Vec<usize>>> {
    let mut out = HashMap::new();
    for (r,iv) in ic.registers_mut().export()?.iter() {
        let iv = Rc::new(iv.copy());
        let v = InterpValue::to_rc_indexes(&iv).map(|x| x.0.to_vec()).unwrap_or(vec![]);
        out.insert(*r,v);
    }
    Ok(out)
}

pub fn std_stream(context: &mut InterpContext) -> anyhow::Result<&mut Stream> {
    let p = context.payload("std","stream")?;
    Ok(p.as_any_mut().downcast_mut::<Stream>().ok_or_else(|| DauphinError::runtime("missing stream"))?)
}

pub fn comp_interpret(is: &CommandInterpretSuite, context: &mut InterpContext, compiler_linker: &CompilerLink, config: &Config, name: &str) -> anyhow::Result<()> {
    let program = compiler_linker.serialize(config)?;
    let interpret_linker = InterpreterLink::new(&is,&program).context("linking")?;
    interpret(context,&interpret_linker,config,name)?;
    Ok(())
}

pub fn interpret(context: &mut InterpContext, interpret_linker: &InterpreterLink, config: &Config, name: &str) -> anyhow::Result<()> {
    let mut interp = interpreter(context,interpret_linker,config,name)?;
    while block_on(interp.more())? {}
    Ok(())
}

pub fn mini_interp_run(context: &mut InterpContext, interpret_linker: &InterpreterLink, config: &Config, name: &str) -> anyhow::Result<()> {
    let start_time = SystemTime::now();
    interpret(context,interpret_linker,config,name)?;
    print!("command time {}ms\n",start_time.elapsed().unwrap_or(Duration::new(0,0)).as_secs_f32()*1000.);
    Ok(())
}

pub fn mini_interp(is: &CommandInterpretSuite, instrs: &Vec<Instruction>, cl: &mut CompilerLink, config: &Config, name: &str) -> anyhow::Result<(HashMap<Register,Vec<usize>>,Vec<String>)> {
    let md = ProgramMetadataBuilder::new(name,None,instrs);
    cl.add(&md,instrs,config)?;
    let program = cl.serialize(config)?;
    let buffer = cbor_serialize(&program)?;
    print!("{}\n",hexdump(&buffer));
    let program = serde_cbor::from_slice(&buffer).context("deserialising")?;
    let interpret_linker = InterpreterLink::new(&is,&program).context("linking")?;
    let mut context = InterpContext::new();
    context.add_payload("std","stream",&ConsoleStreamFactory::new());
    mini_interp_run(&mut context,&interpret_linker,config,name)?;
    let stream = std_stream(&mut context)?;
    let strings = stream.take(0);
    Ok((export_indexes(&mut context)?,strings))
}

pub fn compile(cs: CommandCompileSuite, is: &CommandInterpretSuite, config: &Config, path: &str) -> anyhow::Result<Vec<String>> {
    let mut linker = CompilerLink::new(cs);
    let resolver = common_resolver(&config,&linker)?;
    let mut lexer = Lexer::new(&resolver,"");
    lexer.import(path).expect("cannot load file");
    let mut state = GenerateState::new("test");
    let mut p = Parser::new(&mut state,&mut lexer)?;
    p.parse(&mut state,&mut lexer)?.map_err(|e| DauphinError::runtime(&e.join(". ")))?;
    let stmts = p.take_statements();
    let instrs = generate(&linker,&stmts,&mut state,&resolver,&config,true)?.expect("error");
    let (_,strings) = mini_interp(is,&instrs,&mut linker,&config,"main")?;
    Ok(strings)
}
