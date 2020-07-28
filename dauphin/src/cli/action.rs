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

use anyhow::{ Context };
use rustyline::{ self, Editor };
use dirs::home_dir;
use std::collections::HashMap;
use std::fs::{ write, read };
use std::io::{ self, BufRead, Write };
use std::path::PathBuf;
use std::process::exit;
use regex::Regex;
use crate::suitebuilder::{ make_compiler_suite, make_interpret_suite };
use dauphin_interp::command::InterpreterLink;
use dauphin_interp::stream::{ StreamFactory };
use dauphin_interp::runtime::{ InterpretInstance, DebugInterpretInstance, StandardInterpretInstance, InterpContext };
use dauphin_compile::util::{ fix_filename };
use dauphin_interp::util::DauphinError;
use dauphin_interp::util::cbor::{ cbor_serialize };
use dauphin_compile::lexer::{ Lexer };
use dauphin_compile::parser::{ Parser };
use dauphin_compile::resolver::{ common_resolver, Resolver };
use dauphin_compile::generate::{ generate, GenerateState };
use dauphin_compile::cli::Config;
use dauphin_compile::command::{ CompilerLink, ProgramMetadata, MetaLink, MergeLink };
use serde_cbor::Value as CborValue;
use serde_cbor::to_writer;

pub fn interpreter<'a>(context: &'a mut InterpContext, interpret_linker: &'a InterpreterLink, config: &Config, name: &str) -> anyhow::Result<Box<dyn InterpretInstance + 'a>> {
    if let Some(instrs) = interpret_linker.get_instructions(name)? {
        if config.get_debug_run() {
            return Ok(Box::new(DebugInterpretInstance::new(interpret_linker,&instrs,name,context)?));
        }
    }
    Ok(Box::new(StandardInterpretInstance::new(interpret_linker,name,context)?))
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

fn munge_filename(source: &str) -> &str {
    if let Some(name) = Regex::new(r".*/(.*?)\.dp").unwrap().captures_iter(source).next() {
        name.get(1).unwrap().as_str()
    } else {
        source
    }
}

fn compile_one(config: &Config, resolver: &Resolver, linker: &mut CompilerLink, source: &str, name: &str) -> anyhow::Result<bool> {
    if config.get_verbose() > 0 {
        print!("compiling {}\n",source);
    }
    let mut lexer = Lexer::new(&resolver,name);
    let mut p = Parser::new(&mut lexer)?;
    lexer.import(source)?;
    match p.parse(&mut lexer).context("parsing")? {
        Err(errors) => {
            print!("{}\n",errors.join("\n"));
            return Ok(false);
        },
        Ok(_) => {}
    };
    let stmts = p.take_statements();
    let defstore = p.get_defstore();
    let mut state = GenerateState::new(&defstore);
    let instrs = match generate(&linker,&stmts,&mut state,&resolver,&config).context("generating code")? {
        Err(errors) => {
            print!("{}\n",errors.join("\n"));
            return Ok(false);
        },
        Ok(x) => x
    };
    let note = match config.get_note() {
        "" => None,
        note => Some(note)
    };
    let md = ProgramMetadata::new(&name,note,&instrs);
    linker.add(&md,&instrs,config).context("linking")?;
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
            let path = format!("file:{}",source);
            let filename = munge_filename(source);
            if !compile_one(config,&resolver,&mut linker,&path,&filename).with_context(|| format!("compiling {}",source))? {
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
        let mut context = InterpContext::new();
        let mut sf = StreamFactory::new();
        sf.to_stdout(true);
        context.add_payload("std","stream",&sf);
        for filename in config.get_binary_sources() {
            let suite = make_interpret_suite(config).context("building commands")?;
            let buffer = read_binary_file(filename)?;
            let program = serde_cbor::from_slice(&buffer).context("corrupted binary")?;
            let mut interpret_linker = InterpreterLink::new(&suite,&program).context("linking binary")?;
            let mut interp = interpreter(&mut context,&interpret_linker,&config,config.get_run()).expect("interpreter");
            while interp.more().expect("interpreting") {}
        }
        context.finish();
        Ok(())
    }
}

struct ReplContext<'a,'b> {
    linker: &'b mut CompilerLink,
    config: Config,
    resolver: &'a Resolver,
    lexer: Option<Lexer<'a>>,
    parser: Option<Parser>,
    pending: bool
}

impl<'a,'b> ReplContext<'a,'b> {
    fn new(config: &Config, linker: &'b mut CompilerLink, resolver: &'a Resolver) -> anyhow::Result<ReplContext<'a,'b>> {
        let mut out = ReplContext {
            config: config.clone(),
            linker,
            resolver,
            lexer: None,
            parser: None,
            pending: false
        };
        out.lexer = Some(Lexer::new(&out.resolver,"main"));
        out.parser = Some(Parser::new(out.lexer.as_mut().unwrap())?);
        Ok(out)
    }

    fn generate_more(&mut self) -> anyhow::Result<Option<CborValue>> {
        let s = self.parser.as_mut().unwrap().parse(&mut self.lexer.as_mut().unwrap()).context("parsing")?;
        if !self.lexer.as_ref().unwrap().exhausted() {
            self.pending = true;
            return Ok(None);
        }
        self.pending = false;
        match s {
            Err(errors) => {
                print!("{}\n",errors.join("\n"));
                return Ok(None);
            },
            Ok(_) => {}
        };
        let stmts = self.parser.as_mut().unwrap().take_statements();
        let defstore = self.parser.as_ref().unwrap().get_defstore();
        let mut state = GenerateState::new(&defstore);
        let instrs = match generate(&self.linker,&stmts,&mut state,&self.resolver,&self.config).context("generating code")? {
            Err(errors) => {
                print!("{}\n",errors.join("\n"));
                return Ok(None);
            },
            Ok(x) => x
        };
        let md = ProgramMetadata::new("main",None,&instrs);
        self.linker.add(&md,&instrs,&self.config).context("linking")?;
        Ok(Some(self.linker.serialize(&self.config)?))
    }

    fn pending(&self) -> bool { self.pending }

    fn add_line(&mut self, line: &str) -> anyhow::Result<()> {
        self.lexer.as_mut().unwrap().repl_line(line)
    }
}

struct ReplAction();

fn history_file() -> anyhow::Result<PathBuf> {
    let home_dir = home_dir().ok_or_else(|| DauphinError::integration("could not get home directory"))?;
    Ok(home_dir.join(".dauphin_history"))
}

fn load_history(rl: &mut Editor<()>) -> anyhow::Result<()> {
    rl.load_history(&history_file()?)?;
    Ok(())
}

fn save_history(rl: &mut Editor<()>) -> anyhow::Result<()> {
    rl.save_history(&history_file()?)?;
    Ok(())
}

impl Action for ReplAction {
    fn name(&self) -> String { "repl".to_string() }
    fn execute(&self, config: &Config) -> anyhow::Result<()> {
        let mut context = InterpContext::new();
        let mut sf = StreamFactory::new();
        sf.to_stdout(true);
        context.add_payload("std","stream",&sf);
        let mut rl = rustyline::Editor::<()>::new();
        load_history(&mut rl).unwrap_or(());
        let isuite = make_interpret_suite(config).context("interpreter commands")?;
        let lib = make_compiler_suite(&config).context("registering commands")?;
        let mut clink = CompilerLink::new(lib);
        let resolver = common_resolver(&config,&clink).context("creating file-path resolver")?;
        let mut repl_context = ReplContext::new(config,&mut clink,&resolver)?;
        loop {
            let readline = rl.readline(if repl_context.pending() {"+> " } else { "dauphin> " });
            match readline {
                Ok(line) => {
                    rl.add_history_entry(&line);
                    repl_context.add_line(&line)?;
                    if let Some(program) = repl_context.generate_more().context("compiling")? {
                        let interpret_linker = InterpreterLink::new(&isuite,&program).context("linking binary")?;
                        let mut interp = interpreter(&mut context,&interpret_linker,&config,"main").expect("interpreter");
                        while interp.more().expect("interpreting") {}
                    }
                },
                Err(_) => break
            }
        }
        context.finish();
        save_history(&mut rl).unwrap_or(());
        Ok(())
    }
}

struct ListAction();

impl Action for ListAction {
    fn name(&self) -> String { "list".to_string() }
    fn execute(&self, config: &Config) -> anyhow::Result<()> {
        let mut metalink = vec![];
        for filename in config.get_binary_sources() {
            let buffer = read_binary_file(filename)?;
            let program = serde_cbor::from_slice(&buffer).context("corrupted binary")?;
            metalink.append(&mut MetaLink::new(&program).context("loading metadata")?.ls());
        }
        print!("\n{}\n\n",metalink.join("\n"));
        Ok(())
    }
}

struct MergeAction();

impl Action for MergeAction {
    fn name(&self) -> String { "merge".to_string() }
    fn execute(&self, config: &Config) -> anyhow::Result<()> {
        let ccs = make_compiler_suite(config).context("building commands")?;
        let mut mergelink = MergeLink::new(ccs);
        for filename in config.get_binary_sources() {
            let buffer = read_binary_file(filename)?;
            let data = serde_cbor::from_slice(&buffer).context("corrupted binary")?;
            mergelink.add_file(data).with_context(|| format!("adding {}",filename))?;
        }
        let data = mergelink.serialize().context("serializing")?;
        let buffer = cbor_serialize(&data).context("writing")?;
        write_binary_file(config.get_output(),&buffer).context("writing")?;
        print!("{} written\n",config.get_output());
        Ok(())
    }
}

pub(super) fn make_actions() -> HashMap<String,Box<dyn Action>> {
    let mut out : Vec<Box<dyn Action>> = vec![];
    out.push(Box::new(CompileAction()));
    out.push(Box::new(GenerateDynamicData()));
    out.push(Box::new(RunAction()));
    out.push(Box::new(ListAction()));
    out.push(Box::new(MergeAction()));
    out.push(Box::new(ReplAction()));
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