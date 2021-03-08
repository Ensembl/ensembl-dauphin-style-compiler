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
use crate::cli::Config;
use crate::command::{ CommandCompileSuite, CompilerLink, Instruction };
use crate::core::make_core;
use dauphin_interp::command::{ CommandInterpretSuite, InterpreterLink };
use dauphin_interp::{ make_core_interp };
use dauphin_lib_std::{ make_std_interp };
use dauphin_interp::runtime::{ InterpContext, InterpValue, Register, StandardInterpretInstance, DebugInterpretInstance, InterpretInstance };
use dauphin_interp::stream::{ ConsoleStreamFactory, Stream };
use dauphin_interp::util::cbor::{ cbor_serialize };
use dauphin_lib_std::make_std;
//use dauphin_lib_buildtime::make_buildtime;
use crate::generate::generate;
use crate::resolver::common_resolver;
use crate::lexer::Lexer;
use crate::parser::Parser;
use dauphin_test_harness::find_testdata;

pub fn xxx_test_config() -> Config {
    let mut cfg = Config::new();
    cfg.set_root_dir(&find_testdata().to_string_lossy());
    cfg.set_generate_debug(true);
    cfg.set_unit_test(true);
    cfg.set_verbose(3);
    cfg.set_opt_level(2);
    cfg.set_debug_run(true);
    cfg.add_lib("buildtime");
    cfg.add_file_search_path("*.egs");
    cfg.add_file_search_path("parser/*.egs");
    cfg.add_file_search_path("parser/import-subdir/*.egs");
    cfg
}

pub fn make_interpret_suite() -> anyhow::Result<CommandInterpretSuite> {
    let mut suite = CommandInterpretSuite::new();
    suite.register(make_core_interp())?;
    suite.register(make_std_interp())?;
    Ok(suite)
}

pub fn make_compiler_suite() -> anyhow::Result<CommandCompileSuite> {
    let mut suite = CommandCompileSuite::new();
    suite.register(make_core())?;
    Ok(suite)
}
