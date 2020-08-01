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

use crate::test::{ make_compiler_suite, compile, make_interpret_suite };
use dauphin_test_harness::{ mini_interp, mini_interp_run, load_testdata, comp_interpret, xxx_test_config, compile };
use dauphin_interp::types::{ MemberMode };
use dauphin_interp::command::{ InterpreterLink };
use dauphin_interp::runtime::{ InterpContext };
use dauphin_compile::cli::Config;
use dauphin_compile::resolver::{ common_resolver, Resolver };
use dauphin_compile::parser::{ Parser, parse_type };
use dauphin_compile::lexer::Lexer;
use dauphin_interp::util::DauphinError;
use dauphin_compile::typeinf::{ MemberType, Typing, get_constraint };
use dauphin_compile::command::{ CompilerLink, InstructionType, Instruction, InstructionSuperType };
use dauphin_compile::model::{ DefStore, make_full_type };
use dauphin_compile::generate::{ generate, generate_code, simplify, call, GenerateState };
use dauphin_interp::stream::{ StreamFactory, Stream };

#[test]
fn print_smoke() {
    let config = xxx_test_config();
    let cs = make_compiler_suite(&config).expect("m");
    let is = make_interpret_suite().expect("n");
    let strings = compile(cs,&is,&config,"search:std/print").expect("a");
    for s in &strings {
        print!("{}\n",s);
    }
    assert_eq!(&vec![
        "[print::test3 { A: [[1, 1], [1, 2, 3], [4, 5, 6], [7, 8, 9], [1, 1]], B: [] }, print::test3 { A: [[7], [6], [5]], B: [[4]] }]",
        "[buildtime::version { major: 0, minor: 1 }, buildtime::version { major: 0, minor: 0 }, buildtime::version { major: 0, minor: 0 }]",
        "[print::test { x: [false, true] }, print::test { x: [true, false] }]",
        "[print::test2:A [true, true], print::test2:B [[0], [1, 2, 3]], print::test2:C false, print::test2:A [false]]",
        "1", "2", "3",
        "\'4241030040\'"
    ].iter().map(|x| x.to_string()).collect::<Vec<_>>(),&strings);
}

#[test]
fn assign_filtered() {
    let config = xxx_test_config();
    let cs = make_compiler_suite(&config).expect("m");
    let is = make_interpret_suite().expect("n");
    let strings = compile(cs,&is,&config,"search:std/filterassign").expect("a");
    for s in &strings {
        print!("{}\n",s);
    }
    // XXX todo test it!
}

#[test]
fn assign_shallow() {
    let config = xxx_test_config();
    let cs = make_compiler_suite(&config).expect("m");
    let is = make_interpret_suite().expect("n");
    let strings = compile(cs,&is,&config,"search:std/assignshallow").expect("a");
    for s in &strings {
        print!("{}\n",s);
    }
    print!("{:?}\n",strings);
    assert_eq!("[0, 0]",strings[0]);
}

#[test]
fn extend_smoke() {
    let config = xxx_test_config();
    let mut linker = CompilerLink::new(make_compiler_suite(&config).expect("y"));
    let resolver = common_resolver(&config,&linker).expect("a");
    let mut lexer = Lexer::new(&resolver,"");
    lexer.import("search:std/extend").expect("cannot load file");
    let mut state = GenerateState::new("test");
    let mut p = Parser::new(&mut state,&mut lexer).expect("a");
    p.parse(&mut state,&mut lexer).expect("parse").map_err(|e| DauphinError::runtime(&e.join(". "))).expect("parse");
    let stmts = p.take_statements();
    let instrs = generate(&linker,&stmts,&mut state,&resolver,&config,true).expect("j").expect("k");
    let mut prev : Option<Instruction> = None;
    for instr in &instrs {
        if let InstructionType::Call(id,_,_,_) = &instr.itype {
            if id.name() == "extend" {
                if let Some(prev) = prev {
                    assert_ne!(InstructionSuperType::Pause,prev.itype.supertype().expect("a"));
                }
            }
        }
        prev = Some(instr.clone());
    }
    let is = make_interpret_suite().expect("n");
    let (_,strings) = mini_interp(&is,&instrs,&mut linker,&config,"main").expect("x");
    for s in &strings {
        print!("{}\n",s);
    }
}

#[test]
fn vector_append() {
    let config = xxx_test_config();
    let mut linker = CompilerLink::new(make_compiler_suite(&config).expect("y"));
    let resolver = common_resolver(&config,&linker).expect("a");
    let mut lexer = Lexer::new(&resolver,"");
    lexer.import("search:std/vector-append").expect("cannot load file");
    let mut state = GenerateState::new("test");
    let mut p = Parser::new(&mut state,&mut lexer).expect("a");
    p.parse(&mut state,&mut lexer).expect("parse").map_err(|e| DauphinError::runtime(&e.join(". "))).expect("parse");
    let stmts = p.take_statements();
    let instrs = generate(&linker,&stmts,&mut state,&resolver,&config,true).expect("j").expect("k");
    let is = make_interpret_suite().expect("n");
    let (_,strings) = mini_interp(&is,&instrs,&mut linker,&config,"main").expect("x");
    for s in &strings {
        print!("{}\n",s);
    }
}

#[test]
fn map_smoke() {
    let config = xxx_test_config();
    let mut linker = CompilerLink::new(make_compiler_suite(&config).expect("y"));
    let resolver = common_resolver(&config,&linker).expect("a");
    let mut lexer = Lexer::new(&resolver,"");
    lexer.import("search:std/map").expect("cannot load file");
    let mut state = GenerateState::new("test");
    let mut p = Parser::new(&mut state,&mut lexer).expect("a");
    p.parse(&mut state,&mut lexer).expect("parse").map_err(|e| DauphinError::runtime(&e.join(". "))).expect("parse");
    let stmts = p.take_statements();
    let instrs = generate(&linker,&stmts,&mut state,&resolver,&config,true).expect("j").expect("k");
    let is = make_interpret_suite().expect("n");
    let (_,strings) = mini_interp(&is,&instrs,&mut linker,&config,"main").expect("x");
    for s in &strings {
        print!("{}\n",s);
    }
}
