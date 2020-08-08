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
use std::time::{ SystemTime, Duration };
use std::collections::HashMap;
use std::rc::Rc;
use dauphin_compile::cli::Config;
use dauphin_compile::command::{ CommandCompileSuite, CompilerLink, Instruction };
use dauphin_compile::core::make_core;
use dauphin_interp::command::{ CommandInterpretSuite, InterpreterLink };
use dauphin_interp::{ make_core_interp };
use crate::{ make_std_interp };
use dauphin_interp::runtime::{ InterpContext, InterpValue, Register, StandardInterpretInstance, DebugInterpretInstance, InterpretInstance };
use dauphin_interp::stream::{ StreamFactory, Stream };
use dauphin_interp::util::cbor::{ cbor_serialize };
use crate::make_std;
//use dauphin_lib_buildtime::make_buildtime;
use dauphin_compile::generate::generate;
use dauphin_compile::resolver::common_resolver;
use dauphin_compile::lexer::Lexer;
use dauphin_compile::parser::Parser;
use dauphin_test_harness::hexdump;

pub fn make_interpret_suite() -> anyhow::Result<CommandInterpretSuite> {
    let mut suite = CommandInterpretSuite::new();
    suite.register(make_core_interp())?;
    suite.register(make_std_interp())?;
    Ok(suite)
}

pub fn make_compiler_suite(config: &Config) -> anyhow::Result<CommandCompileSuite> {
    let mut suite = CommandCompileSuite::new();
    suite.register(make_core())?;
    let mut std = make_std();
    let mut sf = StreamFactory::new();
    sf.to_stdout(true);
    std.add_payload("std","stream",sf);
    suite.register(std)?;
    Ok(suite)
}
