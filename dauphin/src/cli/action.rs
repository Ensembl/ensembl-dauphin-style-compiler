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

use std::collections::HashMap;
use std::fmt::Display;
use std::fs::{ write, read };
use std::process::exit;
use regex::Regex;
use crate::suitebuilder::{ make_compiler_suite, make_interpret_suite };
use dauphin_interp::command::InterpreterLink;
use dauphin_lib_std::{ StreamFactory };
use dauphin_interp::runtime::{ InterpretInstance, DebugInterpretInstance, StandardInterpretInstance };
use dauphin_compile::util::{ fix_filename };
use dauphin_interp::util::DauphinError;
use dauphin_interp::util::cbor::{ cbor_serialize };
use dauphin_compile::lexer::{ Lexer };
use dauphin_compile::parser::{ Parser };
use dauphin_compile::resolver::{ common_resolver, Resolver };
use dauphin_compile::generate::generate;
use dauphin_compile::cli::Config;
use dauphin_compile::command::CompilerLink;
use serde_cbor::Value as CborValue;
use serde_cbor::to_writer;

pub fn interpreter<'a>(interpret_linker: &'a InterpreterLink, config: &Config, name: &str) -> anyhow::Result<Box<dyn InterpretInstance<'a> + 'a>> {
    if let Some(instrs) = interpret_linker.get_instructions(name)? {
        if config.get_debug_run() {
            return Ok(Box::new(DebugInterpretInstance::new(interpret_linker,&instrs,name)?));
        }
    }
    Ok(Box::new(StandardInterpretInstance::new(interpret_linker,name)?))
}

fn read_binary_file(filename: &str) -> anyhow::Result<Vec<u8>> {
    read(filename).map_err(|e| DauphinError::OSError(e)).with_context(|| format!("reading {}",filename))
}

fn write_binary_file(filename: &str, contents: &[u8]) -> anyhow::Result<()> {
    write(filename,contents).map_err(|e| DauphinError::OSError(e)).with_context(|| format!("writing {}",filename))
}

fn write_cbor_file(filename: &str, contents: &CborValue) -> anyhow::Result<()> {
    let mut buffer = Vec::new();
    to_writer(&mut buffer,&contents).with_context(|| format!("serializing cbor data"))?;
    write_binary_file(filename,&buffer)?;
    Ok(())
}

pub trait Action {
    fn name(&self) -> String;
    fn execute(&self, config: &Config) -> anyhow::Result<()>;
}

struct VersionAction();

impl Action for VersionAction {
    fn name(&self) -> String { "version".to_string() }
    fn execute(&self, _: &Config) -> anyhow::Result<()> {
        print!("0.0\n");
        Ok(())
    }
}

struct GenerateDynamicData();

impl Action for GenerateDynamicData {
    fn name(&self) -> String { "generate-dynamic-data".to_string() }
    fn execute(&self, config: &Config) -> anyhow::Result<()> {
        let builder = make_compiler_suite(&config).expect("y");
        let linker = CompilerLink::new(builder);
        let data = linker.generate_dynamic_data(&config).expect("x");
        for (suite,data) in data.iter() {
            print!("writing data for {}\n",suite);
            write_cbor_file(&format!("{}.ddd",fix_filename(&suite.to_string())),data)?;
        }
        Ok(())
    }
}

fn compile_one(config: &Config, resolver: &Resolver, linker: &mut CompilerLink, source: &str) -> anyhow::Result<bool> {
    let name = if let Some(name) = Regex::new(r".*/(.*?)\.dp").unwrap().captures_iter(source).next() {
        name.get(1).unwrap().as_str()
    } else {
        source
    };
    if config.get_verbose() > 0 {
        print!("compiling {}\n",source);
    }
    let mut lexer = Lexer::new(&resolver,name);
    lexer.import(&format!("file:{}",source))?;
    let p = Parser::new(&mut lexer);
    let (stmts,defstore) = match p.parse().context("parsing")? {
        Err(errors) => {
            print!("{}\nCompilation failed\n",errors.join("\n"));
            return Ok(false);
        },
        Ok(x) => x
    };
    let instrs = match generate(&linker,&stmts,&defstore,&resolver,&config).context("generating code")? {
        Err(errors) => {
            print!("{}\nCompilation failed\n",errors.join("\n"));
            return Ok(false);
        },
        Ok(x) => x
    };
    linker.add(&name,&instrs,config).context("linking")?;
    Ok(true)
}

struct CompileAction();

impl Action for CompileAction {
    fn name(&self) -> String { "compile".to_string() }
    fn execute(&self, config: &Config) -> anyhow::Result<()> {
        let lib = make_compiler_suite(&config).context("registering commands")?;
        let mut linker = CompilerLink::new(lib);
        let mut sf = StreamFactory::new();
        sf.to_stdout(true);
        linker.add_payload("std","stream",sf);
        let resolver = common_resolver(&config,&linker).context("creating file-path resolver")?;
        let mut emit = true;
        for source in config.get_sources() {
            if !compile_one(config,&resolver,&mut linker,source).with_context(|| format!("compiling {}",source))? {
                emit = false;
            }
        }
        if emit {
            let program = linker.serialize(config).context("serializing")?;
            let buffer = cbor_serialize(&program).context("writing")?;
            write_binary_file(config.get_output(),&buffer)?;
            print!("{} written\n",config.get_output());
        } else {
            print!("did not write output\n");
        }
        Ok(())
    }
}

struct RunAction();

impl Action for RunAction {
    fn name(&self) -> String { "run".to_string() }
    fn execute(&self, config: &Config) -> anyhow::Result<()> {
        let suite = make_interpret_suite(config).context("building commands")?;
        let buffer = read_binary_file(config.get_output())?;
        let program = serde_cbor::from_slice(&buffer).context("corrupted binary")?;
        let mut interpret_linker = InterpreterLink::new(suite,&program).context("linking binary")?;
        let mut sf = StreamFactory::new();
        sf.to_stdout(true);
        interpret_linker.add_payload("std","stream",sf);
        let mut interp = interpreter(&interpret_linker,&config,config.get_run()).expect("interpreter");
        while interp.more().expect("interpreting") {}
        interp.finish();
        Ok(())
    }
}

pub(super) fn make_actions() -> HashMap<String,Box<dyn Action>> {
    let mut out : Vec<Box<dyn Action>> = vec![];
    out.push(Box::new(VersionAction()));
    out.push(Box::new(CompileAction()));
    out.push(Box::new(GenerateDynamicData()));
    out.push(Box::new(RunAction()));
    out.drain(..).map(|a| (a.name(),a)).collect()
}

pub fn run_or_error(config: &Config) -> anyhow::Result<()> {
    config.verify().context("verifying config/options")?;
    let actions = make_actions();
    let action_name = config.get_action();
    if let Some(action) = actions.get(action_name) {
        action.execute(config)?;
    } else {
        eprint!("Invalid action '{}'\n",action_name);
    }
    Ok(())
}

pub fn run(config: &Config) {
    match run_or_error(config) {
        Ok(_) => {},
        Err(e) => {
            eprint!("Error: {:?}\n",e);
            exit(2);
        }
    }
}