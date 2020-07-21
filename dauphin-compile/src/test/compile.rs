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
use std::time::{ SystemTime, Duration };
use std::collections::HashMap;
use std::rc::Rc;
use crate::cli::Config;
use crate::command::{ CommandCompileSuite, CompilerLink, Instruction, ProgramMetadata };
use dauphin_interp::command::{ CommandInterpretSuite, InterpreterLink };
use dauphin_interp::{ make_core_interp };
use dauphin_lib_std::{ make_std_interp };
use dauphin_interp::runtime::{ InterpContext, InterpValue, Register };
use dauphin_lib_std::stream::{ StreamFactory, Stream };
use dauphin_interp::util::cbor::{ cbor_serialize };
use dauphin_interp::util::{ DauphinError };
use crate::core::{ make_core };
use crate::generate::generate;
use crate::resolver::common_resolver;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::test::cbor::hexdump;
use dauphin_interp::runtime::{ StandardInterpretInstance, DebugInterpretInstance, InterpretInstance };

pub fn interpreter<'a>(interpret_linker: &'a InterpreterLink, config: &Config, name: &str) -> anyhow::Result<Box<dyn InterpretInstance<'a> + 'a>> {
    if let Some(instrs) = interpret_linker.get_instructions(name)? {
        if config.get_debug_run() {
            return Ok(Box::new(DebugInterpretInstance::new(interpret_linker,&instrs,name)?));
        }
    }
    Ok(Box::new(StandardInterpretInstance::new(interpret_linker,name)?))
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
    Ok(p.as_any_mut().downcast_mut().ok_or_else(|| DauphinError::runtime("no stream context"))?)
}

pub fn comp_interpret(compiler_linker: &CompilerLink, config: &Config, name: &str) -> anyhow::Result<InterpContext> {
    let program = compiler_linker.serialize(config)?;
    let mut interpret_linker = InterpreterLink::new(make_interpret_suite()?,&program).context("linking")?;
    interpret_linker.add_payload("std","stream",StreamFactory::new()); 
    interpret(&interpret_linker,config,name)
}

pub fn interpret(interpret_linker: &InterpreterLink, config: &Config, name: &str) -> anyhow::Result<InterpContext> {
    let mut interp = interpreter(interpret_linker,config,name)?;
    while interp.more()? {}
    Ok(interp.finish())
}

pub fn mini_interp_run(interpret_linker: &InterpreterLink, config: &Config, name: &str) -> anyhow::Result<InterpContext> {
    let interp = interpreter(interpret_linker,config,name)?;
    let start_time = SystemTime::now();
    let out = interpret(interpret_linker,config,name)?;
    print!("command time {}ms\n",start_time.elapsed().unwrap_or(Duration::new(0,0)).as_secs_f32()*1000.);
    Ok(out)
}

pub fn mini_interp(instrs: &Vec<Instruction>, cl: &mut CompilerLink, config: &Config, name: &str) -> anyhow::Result<(HashMap<Register,Vec<usize>>,Vec<String>)> {
    let md = ProgramMetadata::new(name,None,instrs);
    cl.add(&md,instrs,config)?;
    let program = cl.serialize(config)?;
    let buffer = cbor_serialize(&program)?;
    print!("{}\n",hexdump(&buffer));
    let suite = make_interpret_suite()?;
    let program = serde_cbor::from_slice(&buffer).context("deserialising")?;
    let mut interpret_linker = InterpreterLink::new(suite,&program).context("linking")?;
    interpret_linker.add_payload("std","stream",StreamFactory::new());
    let mut ic = mini_interp_run(&interpret_linker,config,name)?;
    let stream = std_stream(&mut ic)?;
    let strings = stream.take();
    Ok((export_indexes(&mut ic)?,strings))
}

pub fn make_interpret_suite() -> anyhow::Result<CommandInterpretSuite> {
    let mut suite = CommandInterpretSuite::new();
    suite.register(make_core_interp())?;
    suite.register(make_std_interp())?;
    Ok(suite)
}

pub fn make_compiler_suite(config: &Config) -> anyhow::Result<CommandCompileSuite> {
    let mut suite = CommandCompileSuite::new();
    suite.register(make_core())?;
    Ok(suite)
}

pub fn compile(config: &Config, path: &str) -> anyhow::Result<Vec<String>> {
    let mut linker = CompilerLink::new(make_compiler_suite(&config)?);
    let resolver = common_resolver(&config,&linker)?;
    let mut lexer = Lexer::new(&resolver,"");
    lexer.import(path).expect("cannot load file");
    let p = Parser::new(&mut lexer);
    let (stmts,defstore) = p.parse()?.map_err(|e| DauphinError::runtime(&e.join(". ")))?;
    let instrs = generate(&linker,&stmts,&defstore,&resolver,&config)?.expect("errors");
    let (_,strings) = mini_interp(&instrs,&mut linker,&config,"main")?;
    Ok(strings)
}
