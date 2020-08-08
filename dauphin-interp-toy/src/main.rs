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
use std::fmt::Display;
use std::fs::read;
use std::process::exit;
use serde_cbor;
use dauphin_interp::make_core_interp;
use dauphin_interp::command::{ CommandInterpretSuite, InterpreterLink };
use dauphin_interp::runtime::{ PartialInterpretInstance, InterpretInstance, InterpContext };
use dauphin_lib_std::{ make_std_interp };
use dauphin_interp::stream::{ StreamFactory };

fn make_suite() -> anyhow::Result<CommandInterpretSuite> {
    let mut suite = CommandInterpretSuite::new();
    suite.register(make_core_interp()).context("registering core")?;
    suite.register(make_std_interp()).context("registering std")?;
    Ok(suite)     
}

fn main_real() -> anyhow::Result<()> {
    let binary_file = String::new();
    let name = String::new();

    let suite = make_suite()?;
    let buffer = read(&binary_file).with_context(|| format!("reading {}",binary_file))?;
    let program = serde_cbor::from_slice(&buffer).context("while deserialising")?;
    let mut linker = InterpreterLink::new(&suite,&program)?;
    let mut context = InterpContext::new();
    let mut sf = StreamFactory::new();
    sf.to_stdout(true);
    context.add_payload("std","stream",&sf);
    let mut interp = Box::new(PartialInterpretInstance::new(&linker,&name,&mut context)).context("building interpreter")?;
    while interp.more().expect("interpreting") {}
    context.finish();
    Ok(())
}

fn main() {
    match main_real() {
        Ok(options) => {},
        Err(error) => {
            eprint!("Error: {}\n",error);
        }
    }
}
