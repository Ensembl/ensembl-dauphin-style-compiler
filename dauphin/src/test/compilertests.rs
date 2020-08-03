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

use crate::test::{ make_compiler_suite, make_interpret_suite };
use dauphin_test_harness::{ xxx_test_config, compile, load_testdata, mini_interp, comp_interpret, mini_interp_run };
use dauphin_interp::types::{ MemberMode };
use dauphin_interp::command::{ InterpreterLink };
use dauphin_interp::runtime::InterpContext;
use dauphin_interp::util::DauphinError;
use dauphin_compile::cli::Config;
use dauphin_compile::resolver::{ common_resolver, Resolver };
use dauphin_compile::parser::{ Parser, parse_type };
use dauphin_compile::lexer::Lexer;
use dauphin_compile::typeinf::{ MemberType, Typing, get_constraint };
use dauphin_compile::command::{ CompilerLink, InstructionType, ProgramMetadata };
use dauphin_compile::model::{ DefStore, make_full_type };
use dauphin_compile::generate::{ generate, generate_code, simplify, call, GenerateState };
use dauphin_interp::stream::{ StreamFactory, Stream };

// XXX move to common test utils
fn make_type(defstore: &DefStore, name: &str) -> MemberType {
    let config = xxx_test_config();
    let linker = CompilerLink::new(make_compiler_suite(&config).expect("y"));
    let resolver = common_resolver(&config,&linker).expect("a");
    let mut lexer = Lexer::new(&resolver,"");
    lexer.import(&format!("data:{}",name)).expect("cannot load file");
    parse_type(&mut lexer,defstore).expect("bad type")
}

fn load_cmp(filename: &str) -> String {
    let outdata = load_testdata(&["codegen",filename]).ok().unwrap();
    let mut seq = vec![];
    for line in outdata.split("\n") {
        if line.starts_with("+") {
            if let Some(part) = line.split_ascii_whitespace().nth(1) {
                seq.push(part);
            }
        }
    }
    seq.join(",")
}

#[test]
fn offset_enums() {
    let config = xxx_test_config();
    let mut linker = CompilerLink::new(make_compiler_suite(&config).expect("y"));
    let resolver = common_resolver(&config,&linker).expect("a");
    let mut lexer = Lexer::new(&resolver,"");
    lexer.import("search:codegen/offset-enums").expect("cannot load file");
    let mut state = GenerateState::new("test");
    let mut p = Parser::new(&mut state,&mut lexer).expect("a");
    p.parse(&mut state,&mut lexer).expect("parse").map_err(|e| DauphinError::runtime(&e.join(". "))).expect("parse");
    let stmts = p.take_statements();
    let regs = make_full_type(state.defstore(),MemberMode::In,&make_type(&state.defstore(),"offset_enums::stest")).expect("b");
    assert_eq!(load_cmp("offset-enums.out"),regs.to_string());
    let instrs = generate(&linker,&stmts,&mut state,&resolver,&config,true).expect("m").expect("errors");
    let is = make_interpret_suite().expect("m");
    let (_,strings) = mini_interp(&is,&instrs,&mut linker,&config,"main").expect("x");
    for s in &strings {
        print!("{}\n",s);
    }
}

#[test]
fn typing_smoke() {
    let mut config = xxx_test_config();
    config.set_opt_seq("");
    let linker = CompilerLink::new(make_compiler_suite(&config).expect("y"));
    let resolver = common_resolver(&config,&linker).expect("cfg");
    let mut lexer = Lexer::new(&resolver,"");
    lexer.import("search:codegen/typepass-smoke").expect("cannot load file");
    let mut state = GenerateState::new("test");
    let mut p = Parser::new(&mut state,&mut lexer).expect("a");
    p.parse(&mut state,&mut lexer).expect("parse").map_err(|e| DauphinError::runtime(&e.join(". "))).expect("parse");
    let stmts = p.take_statements();
    let instrs = generate(&linker,&stmts,&mut state,&resolver,&config,true).expect("m").expect("errors");
    let instrs_str : Vec<String> = instrs.iter().map(|v| format!("{:?}",v)).collect();
    print!("{}\n",instrs_str.join(""));
    let mut tp = Typing::new();
    for instr in &instrs {
        print!("=== {:?}",instr);
        tp.add(&get_constraint(&instr,state.defstore()).expect("A")).expect("ok");
        print!("{:?}\n",tp);
    }
}

// XXX test pruning, eg fewer lines
#[test]
fn assign_regs_smoke() {
    let mut config = xxx_test_config();
    config.set_opt_seq("pca");
    let cs = make_compiler_suite(&config).expect("n");
    let is = make_interpret_suite().expect("m");
    let strings = compile(cs,&is,&config,"search:codegen/linearize-refsquare").expect("a");
    for s in &strings {
        print!("{}\n",s);
    }
    assert_eq!(vec!["[[0], [2], [0], [4]]", "[[0], [2], [9, 9, 9], [9, 9, 9]]", "[0, 0, 0]", "[[0], [2], [8, 9, 9], [9, 9, 9]]"],strings);
}

#[test]
fn call_smoke() {
    let config = xxx_test_config();
    let cs = make_compiler_suite(&config).expect("n");
    let is = make_interpret_suite().expect("m");
    let strings = compile(cs,&is,&config,"search:codegen/module-smoke").expect("a");
    for s in &strings {
        print!("{}\n",s);
    }
}

#[test]
fn lvalue_regression() {
    let config = xxx_test_config();
    let cs = make_compiler_suite(&config).expect("n");
    let is = make_interpret_suite().expect("m");
    let strings = compile(cs,&is,&config,"search:codegen/lvalue").expect("a");
    for s in &strings {
        print!("{}\n",s);
    }
    assert_eq!(vec!["1","2","33"],strings);
}

#[test]
fn line_number_smoke() {
    let mut config = xxx_test_config();
    config.set_opt_seq("");
    let mut linker = CompilerLink::new(make_compiler_suite(&config).expect("y"));
    let resolver = common_resolver(&config,&linker).expect("a");
    let mut lexer = Lexer::new(&resolver,"");
    lexer.import("search:codegen/line-number").expect("cannot load file");
    let mut state = GenerateState::new("test");
    let mut p = Parser::new(&mut state,&mut lexer).expect("a");
    p.parse(&mut state,&mut lexer).expect("parse").map_err(|e| DauphinError::runtime(&e.join(". "))).expect("parse");
    let stmts = p.take_statements();
    let instrs = generate(&linker,&stmts,&mut state,&resolver,&config,true).expect("m").expect("errors");
    let md = ProgramMetadata::new("main",None,&instrs);
    linker.add(&md,&instrs,&config).expect("a");
    let mut context = context();
    let is = make_interpret_suite().expect("m");
    let message = comp_interpret(&is,&mut context,&mut linker,&config,"main").map(|_| ()).expect_err("x").to_string();
    print!("{}\n",message);
    assert!(message.contains("line-number.dp:10"));
}

#[test]
fn no_line_number_smoke() {
    let mut config = xxx_test_config();
    config.set_generate_debug(false);
    config.set_opt_seq("");
    let mut linker = CompilerLink::new(make_compiler_suite(&config).expect("y"));
    let resolver = common_resolver(&config,&linker).expect("a");
    let mut lexer = Lexer::new(&resolver,"");
    lexer.import("search:codegen/line-number").expect("cannot load file");
    let mut state = GenerateState::new("test");
    let mut p = Parser::new(&mut state,&mut lexer).expect("a");
    p.parse(&mut state,&mut lexer).expect("parse").map_err(|e| DauphinError::runtime(&e.join(". "))).expect("parse");
    let stmts = p.take_statements();
    let instrs = generate(&linker,&stmts,&mut state,&resolver,&config,true).expect("m").expect("errors");
    let md = ProgramMetadata::new("main",None,&instrs);
    linker.add(&md,&instrs,&config).expect("a");
    let mut context = context();
    let is = make_interpret_suite().expect("m");
    let message = comp_interpret(&is,&mut context,&mut linker,&config,"main").map(|_| ()).expect_err("x").to_string();
    print!("{}\n",message);
    assert!(!message.contains(" at "));
}

#[test]
fn runnums_smoke() {
    let config = xxx_test_config();
    let cs = make_compiler_suite(&config).expect("n");
    let is = make_interpret_suite().expect("m");
    let strings = compile(cs,&is,&config,"search:codegen/linearize-refsquare").expect("a");
    for s in &strings {
        print!("{}\n",s);
    }
    assert_eq!(vec!["[[0], [2], [0], [4]]", "[[0], [2], [9, 9, 9], [9, 9, 9]]", "[0, 0, 0]", "[[0], [2], [8, 9, 9], [9, 9, 9]]"],strings);
}

#[test]
fn size_hint() {
    let mut config = xxx_test_config();
    config.set_generate_debug(false);
    let cs = make_compiler_suite(&config).expect("n");
    let is = make_interpret_suite().expect("m");
    let strings = compile(cs,&is,&config,"search:codegen/size-hint").expect("a");
    assert_eq!(vec!["\"hello world!\"", "1", "1", "3", "2", "2", "1000000000", "1000000000", "1000000000", "1000000000", "1000000000", "10", "10", "10", "1", "11", "11", "11"],strings);
    print!("{:?}\n",strings);
}

// XXX common
fn compare_instrs(a: &Vec<String>,b: &Vec<String>) {
    print!("compare:\nLHS\n{:?}\n\nRHS\n{:?}\n",a.join("\n"),b.join("\n"));
    let mut a_iter = a.iter();
    for (i,b) in b.iter().enumerate() {
        if let Some(a) = a_iter.next() {
            let a = a.trim();
            let b = b.trim();
            assert_eq!(a,b,"mismatch {:?} {:?} line {}",a,b,i);
        } else if b != "" {
            panic!("premature eof lhs");
        }
    }
    if a_iter.next().is_some() {
        panic!("premature eof rhs");
    }
}

#[test]
fn simplify_smoke() {
    let config = xxx_test_config();
    let linker = CompilerLink::new(make_compiler_suite(&config).expect("y"));
    let resolver = common_resolver(&config,&linker).expect("a");
    let mut lexer = Lexer::new(&resolver,"");
    lexer.import("search:codegen/simplify-smoke").expect("cannot load file");
    let mut state = GenerateState::new("test");
    let mut p = Parser::new(&mut state,&mut lexer).expect("a");
    p.parse(&mut state,&mut lexer).expect("parse").map_err(|e| DauphinError::runtime(&e.join(". "))).expect("parse");
    let stmts = p.take_statements();
    let mut context = generate_code(&mut state,&stmts,true).expect("codegen").expect("m");
    call(&mut context).expect("j");
    simplify(&mut context).expect("k");
    let outdata = load_testdata(&["codegen","simplify-smoke.out"]).ok().unwrap();
    let cmds : Vec<String> = context.get_instructions().iter().map(|e| format!("{:?}",e)).collect();
    compare_instrs(&cmds,&outdata.split("\n").map(|x| x.to_string()).collect());
}

#[test]
fn simplify_enum_nest() {
    let config = xxx_test_config();
    let cs = make_compiler_suite(&config).expect("n");
    let is = make_interpret_suite().expect("m");
    compile(cs,&is,&config,"search:codegen/simplify-enum-nest").expect("a");    
}

#[test]
fn simplify_enum_lvalue() {
    let config = xxx_test_config();
    let mut linker = CompilerLink::new(make_compiler_suite(&config).expect("y"));
    let resolver = common_resolver(&config,&linker).expect("a");
    let mut lexer = Lexer::new(&resolver,"");
    lexer.import("search:codegen/enum-lvalue").expect("cannot load file");
    let mut state = GenerateState::new("test");
    let mut p = Parser::new(&mut state,&mut lexer).expect("a");
    p.parse(&mut state,&mut lexer).expect("parse").map_err(|e| DauphinError::runtime(&e.join(". "))).expect("parse");
    let stmts = p.take_statements();
    let instrs = generate(&linker,&stmts,&mut state,&resolver,&config,true).expect("m").expect("errors");
    let is = make_interpret_suite().expect("m");
    let (_,strings) = mini_interp(&is,&instrs,&mut linker,&config,"main").expect("x");
    for s in &strings {
        print!("{}\n",s);
    }  
}

#[test]
fn simplify_struct_lvalue() {
    let config = xxx_test_config();
    let mut linker = CompilerLink::new(make_compiler_suite(&config).expect("y"));
    let resolver = common_resolver(&config,&linker).expect("a");
    let mut lexer = Lexer::new(&resolver,"");
    lexer.import("search:codegen/struct-lvalue").expect("cannot load file");
    let mut state = GenerateState::new("test");
    let mut p = Parser::new(&mut state,&mut lexer).expect("a");
    p.parse(&mut state,&mut lexer).expect("parse").map_err(|e| DauphinError::runtime(&e.join(". "))).expect("parse");
    let stmts = p.take_statements();
    let instrs = generate(&linker,&stmts,&mut state,&resolver,&config,true).expect("m").expect("errors");
    print!("{:?}",instrs.iter().map(|x| format!("{:?}",x)).collect::<Vec<_>>().join(""));
    let is = make_interpret_suite().expect("m");
    let (_,strings) = mini_interp(&is,&instrs,&mut linker,&config,"main").expect("x");
    for s in &strings {
        print!("{}\n",s);
    }
}

#[test]
fn simplify_both_lvalue() {
    let config = xxx_test_config();
    let mut linker = CompilerLink::new(make_compiler_suite(&config).expect("y"));
    let resolver = common_resolver(&config,&linker).expect("a");
    let mut lexer = Lexer::new(&resolver,"");
    lexer.import("search:codegen/both-lvalue").expect("cannot load file");
    let mut state = GenerateState::new("test");
    let mut p = Parser::new(&mut state,&mut lexer).expect("a");
    p.parse(&mut state,&mut lexer).expect("parse").map_err(|e| DauphinError::runtime(&e.join(". "))).expect("parse");
    let stmts = p.take_statements();
    let instrs = generate(&linker,&stmts,&mut state,&resolver,&config,true).expect("m").expect("errors");
    print!("{:?}",instrs.iter().map(|x| format!("{:?}",x)).collect::<Vec<_>>().join(""));
    let is = make_interpret_suite().expect("m");
    let (_,strings) = mini_interp(&is,&instrs,&mut linker,&config,"main").expect("x");
    for s in &strings {
        print!("{}\n",s);
    }  
}

#[test]
fn dealias_smoke() {
    // XXX check all aliases gone
    let config = xxx_test_config();
    let mut linker = CompilerLink::new(make_compiler_suite(&config).expect("y"));
    let resolver = common_resolver(&config,&linker).expect("a");
    let mut lexer = Lexer::new(&resolver,"");
    lexer.import("search:codegen/linearize-refsquare").expect("cannot load file");
    let mut state = GenerateState::new("test");
    let mut p = Parser::new(&mut state,&mut lexer).expect("a");
    p.parse(&mut state,&mut lexer).expect("parse").map_err(|e| DauphinError::runtime(&e.join(". "))).expect("parse");
    let stmts = p.take_statements();
    let instrs = generate(&linker,&stmts,&mut state,&resolver,&config,true).expect("m").expect("errors");
    let is = make_interpret_suite().expect("m");
    let (values,strings) = mini_interp(&is,&instrs,&mut linker,&config,"main").expect("x");
    print!("{:?}\n",values);
    for s in &strings {
        print!("{}\n",s);
    }
    for instr in &instrs {
        if let InstructionType::Alias = instr.itype {
            assert!(false);
        }
    }
    assert_eq!(vec!["[[0], [2], [0], [4]]", "[[0], [2], [9, 9, 9], [9, 9, 9]]", "[0, 0, 0]", "[[0], [2], [8, 9, 9], [9, 9, 9]]"],strings);
}

#[test]
fn reuse_regs_smoke() {
    let config = xxx_test_config();
    let mut linker = CompilerLink::new(make_compiler_suite(&config).expect("y"));
    let resolver = common_resolver(&config,&linker).expect("a");
    let mut lexer = Lexer::new(&resolver,"");
    lexer.import("search:codegen/reuse-regs").expect("cannot load file");
    let mut state = GenerateState::new("test");
    let mut p = Parser::new(&mut state,&mut lexer).expect("a");
    p.parse(&mut state,&mut lexer).expect("parse").map_err(|e| DauphinError::runtime(&e.join(". "))).expect("parse");
    let stmts = p.take_statements();
    let instrs = generate(&linker,&stmts,&mut state,&resolver,&config,true).expect("m").expect("errors");
    print!("{:?}",instrs.iter().map(|x| format!("{:?}",x)).collect::<Vec<_>>().join(""));
    let is = make_interpret_suite().expect("m");
    let (_,strings) = mini_interp(&is,&instrs,&mut linker,&config,"main").expect("x");
    for s in &strings {
        print!("{}\n",s);
    }
    let mut lt = 0;
    for instr in instrs {
        if let InstructionType::Call(id,_,_,_) = &instr.itype {
            if id.name() == "lt" { lt += 1; }
        }
    }
    assert_eq!(1,lt);
}

fn pause_check(filename: &str) -> bool {
    let mut config = xxx_test_config();
    config.set_generate_debug(false);
    config.set_opt_seq("pcpmuedpdpa"); /* no r to avoid re-ordering */
    let linker = CompilerLink::new(make_compiler_suite(&config).expect("y"));
    let resolver = common_resolver(&config,&linker).expect("a");
    let mut lexer = Lexer::new(&resolver,"");
    let mut state = GenerateState::new("test");
    lexer.import(&format!("search:codegen/{}",filename)).expect("cannot load file");
    let mut p = Parser::new(&mut state,&mut lexer).expect("a");
    p.parse(&mut state,&mut lexer).expect("parse").map_err(|e| DauphinError::runtime(&e.join(". "))).expect("parse");
    let stmts = p.take_statements();
    let instrs = generate(&linker,&stmts,&mut state,&resolver,&config,true).expect("m").expect("errors");
    let mut seen_force_pause = false;
    for instr in &instrs {
        if seen_force_pause {
            print!("AFTER {:?}",instr);
            return if let InstructionType::Pause(_) = &instr.itype {
                true
            } else {
                false
            };
        }
        if let InstructionType::Pause(true) = &instr.itype {
            seen_force_pause = true;
        }
    }
    false
}

#[test]
fn pause() {
    assert!(pause_check("pause"));
    assert!(!pause_check("no-pause"));
}

fn make_program(linker: &mut CompilerLink, resolver: &Resolver, config: &Config, name: &str, path: &str) -> anyhow::Result<()> {
    let mut lexer = Lexer::new(&resolver,"");
    lexer.import(path).expect("cannot load file");
    let mut state = GenerateState::new("test");
    let mut p = Parser::new(&mut state,&mut lexer).expect("a");
    p.parse(&mut state,&mut lexer).expect("parse").map_err(|e| DauphinError::runtime(&e.join(". "))).expect("parse");
    let stmts = p.take_statements();
    let instrs = generate(&linker,&stmts,&mut state,&resolver,&config,true).expect("m").expect("errors");
    let md = ProgramMetadata::new(name,None,&instrs);
    linker.add(&md,&instrs,config)?;
    Ok(())
}

pub fn std_stream(context: &mut InterpContext) -> anyhow::Result<&mut Stream> {
    let p = context.payload("std","stream")?;
    Ok(p.as_any_mut().downcast_mut().ok_or_else(|| DauphinError::runtime("No stream context"))?)
}    

fn context() -> InterpContext {
    let mut ctx = InterpContext::new();
    ctx.add_payload("std","stream",&StreamFactory::new());
    ctx
}

#[test]
fn test_multi_program() {
    let mut config = xxx_test_config();
    config.set_generate_debug(false);
    config.set_verbose(2);
    let mut linker = CompilerLink::new(make_compiler_suite(&config).expect("y"));
    let resolver = common_resolver(&config,&linker).expect("a");
    make_program(&mut linker,&resolver,&config,"prog1","search:codegen/multiprog1").expect("cannot build prog1");
    make_program(&mut linker,&resolver,&config,"prog2","search:codegen/multiprog2").expect("cannot build prog2");
    make_program(&mut linker,&resolver,&config,"prog3","search:codegen/multiprog3").expect("cannot build prog3");
    let program = linker.serialize(&config).expect("serialize");
    let suite = make_interpret_suite().expect("c");
    let mut interpret_linker = InterpreterLink::new(&suite,&program).map_err(|x| format!("{} while linking",x)).expect("d");
    let mut ic_a = context();
    let mut ic_b = context();
    mini_interp_run(&mut ic_a,&interpret_linker,&config,"prog2").expect("A");
    mini_interp_run(&mut ic_b,&interpret_linker,&config,"prog1").expect("B");
    let s_a = std_stream(&mut ic_a).expect("d");
    let s_b = std_stream(&mut ic_b).expect("e");
    let a = &s_a.take(0);
    let b = &s_b.take(0);    
    assert_eq!(vec!["prog2"],a.iter().map(|x| x).collect::<Vec<_>>());
    assert_eq!(vec!["prog1"],b.iter().map(|x| x).collect::<Vec<_>>());
}

#[test]
fn linearize_smoke() {
    let config = xxx_test_config();
    let mut linker = CompilerLink::new(make_compiler_suite(&config).expect("y"));
    let resolver = common_resolver(&config,&linker).expect("a");
    let mut lexer = Lexer::new(&resolver,"");
    lexer.import("search:codegen/linearize-smoke").expect("cannot load file");
    let mut state = GenerateState::new("test");
    let mut p = Parser::new(&mut state,&mut lexer).expect("a");
    p.parse(&mut state,&mut lexer).expect("error").expect("error");
    let stmts = p.take_statements();
    let mut context = generate_code(&mut state,&stmts,true).expect("codegen").expect("success");
    print!("{:?}\n",context);
    let is = make_interpret_suite().expect("m");
    let values = mini_interp(&is,&context.get_instructions(),&mut linker,&config,"main");
    print!("{:?}",values);
}
