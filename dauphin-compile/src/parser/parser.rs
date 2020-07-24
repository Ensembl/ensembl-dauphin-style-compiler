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
use std::mem::replace;
use crate::lexer::{ Lexer, Token };
use crate::model::DefStore;
use super::node::{ Statement, ParserStatement };
use super::declare::declare;
use super::parsestmt::{ parse_statement };
use dauphin_interp::util::{ DauphinError, triage_source_errors };

pub struct Parser {
    defstore: DefStore,
    stmts: Vec<Statement>
}

impl Parser {
    pub fn new(lexer: &mut Lexer) -> anyhow::Result<Parser> {
        let source = lexer.get_source().to_string();
        let mut p = Parser {
            defstore: DefStore::new(&source),
            stmts: vec![]
        };
        lexer.import("preamble:").ok();
        p.parse(lexer).and_then(|out| out.map_err(|e| {
            let err = format!("parsing preamble: {}",e.join(". ").to_string());
            DauphinError::internal(file!(),line!()).context(err)
        }))?;
        Ok(p)
    }

    fn ffwd_error(&mut self, lexer: &mut Lexer) {
        loop {
            match lexer.get() {
                Token::Other(';') => return,
                Token::EndOfLex => return,
                _ => ()
            }
        }
    }

    fn benign_error(&self, e: &anyhow::Error) -> bool {
        if let Some(e) = e.downcast_ref::<DauphinError>() {
            match e {
                DauphinError::SourceError(_,_,_) | DauphinError::FloatingSourceError(_) => {
                    return true;
                },
                _ => {}
            }
        }
        false
    }

    fn recover_parse_statement(&mut self, lexer: &mut Lexer) -> Result<Vec<ParserStatement>,(anyhow::Error,bool)> {
        parse_statement(lexer,&self.defstore,false).map_err(|e| {
            if self.benign_error(&e) {
                self.ffwd_error(lexer);
                (e,true)
            } else {
                (e,false)
            }
        })
    }

    fn run_declare(&mut self, lexer: &mut Lexer, stmt: &ParserStatement) -> anyhow::Result<bool> {
        declare(&stmt,lexer,&mut self.defstore)
    }

    fn get_non_declare(&mut self, lexer: &mut Lexer) -> Result<Vec<ParserStatement>,(anyhow::Error,bool)> {
        let mut out = vec![];
        match self.recover_parse_statement(lexer) {
            Ok(stmts) => {
                for stmt in stmts.iter() {
                    if !self.run_declare(lexer,stmt).map_err(|e| {
                        let benign = self.benign_error(&e);
                        (e,benign)
                    })? {
                        out.push(stmt.clone());
                    }
                }
            },
            Err(e) => { return Err(e); }
        }
        Ok(out)
    }

    fn parse_return(&self, mut errors: Vec<anyhow::Error>) -> anyhow::Result<Result<(),Vec<String>>> {
        if errors.len() > 0 {
            return match triage_source_errors(&mut errors) {
                Ok(e) => Ok(Err(e)),
                Err(e) => Err(e)
            };
        } else {
            return Ok(Ok(()))
        }
    }

    pub fn parse(&mut self, lexer: &mut Lexer) -> anyhow::Result<Result<(),Vec<String>>> {
        let mut errors = vec![];
        loop {
            match self.get_non_declare(lexer) {
                Ok(mut stmts) => {
                    for stmt in stmts.drain(..) {
                        match stmt {
                            ParserStatement::EndOfParse => { return self.parse_return(errors); },
                            ParserStatement::Regular(stmt) => { self.stmts.push(stmt); }
                            _ => {},
                        }
                    }
                },
                Err((error,true)) => { errors.push(error); }
                Err((error,false)) => { return Err(error); }
            }
        }
    }

    pub fn get_defstore(&self) -> &DefStore { &self.defstore }
    pub fn take_statements(&mut self) -> Vec<Statement> { replace(&mut self.stmts,vec![]) }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::resolver::common_resolver;
    use dauphin_interp::command::Identifier;
    use dauphin_interp::util::DauphinError;
    use dauphin_test_harness::{ load_testdata };
    use crate::test::{ make_compiler_suite, xxx_test_config };
    use crate::command::CompilerLink;

    fn last_statement(p: &mut Parser, lexer: &mut Lexer) -> anyhow::Result<ParserStatement> {
        let mut prev = Err(DauphinError::runtime("unexpected EOF"));
        loop {
            let mut stmts = p.recover_parse_statement(lexer).map_err(|e| e.0)?;
            match stmts.pop() {
                Some(ParserStatement::EndOfParse) => break,
                Some(x) => prev = Ok(x),
                None => ()
            }
        }
        return prev;
    }

    #[test]
    fn statement() {
        let config = xxx_test_config();
        let linker = CompilerLink::new(make_compiler_suite().expect("y"));
        let resolver = common_resolver(&config,&linker).expect("a");
        let mut lexer = Lexer::new(&resolver,"");
        let mut p = Parser::new(&mut lexer).expect("k");
        lexer.import("data: import \"x\";").ok();
        assert_eq!(ParserStatement::Import("x".to_string()),last_statement(&mut p,&mut lexer).expect("w"));
    }

    #[test]
    fn import_statement() {
        let config = xxx_test_config();
        let linker = CompilerLink::new(make_compiler_suite().expect("y"));
        let resolver = common_resolver(&config,&linker).expect("a");
        let mut lexer = Lexer::new(&resolver,"");
        let mut p = Parser::new(&mut lexer).expect("k");
        lexer.import("data: import \"data: $;\";").ok();
        let err = p.parse(&mut lexer).ok().unwrap().expect_err("success");
        assert_eq!("data: $;:1 $ encountered outside filter".to_string(),err[0]);
    }

    #[test]
    fn import_search_statement() {
        let config = xxx_test_config();
        let linker = CompilerLink::new(make_compiler_suite().expect("y"));
        let resolver = common_resolver(&config,&linker).expect("A");
        let mut lexer = Lexer::new(&resolver,"");
        let mut p = Parser::new(&mut lexer).expect("k");
        lexer.import("search:import-search").expect("cannot load file");
        let txt = "import-smoke4.dp:1 Reserved keyword \'reserved\' found";
        assert_eq!(txt,p.parse(&mut lexer).ok().unwrap().expect_err("x")[0]);
    }

    #[test]
    fn test_preprocess() {
        let config = xxx_test_config();
        let linker = CompilerLink::new(make_compiler_suite().expect("y"));
        let resolver = common_resolver(&config,&linker).expect("a");
        let mut lexer = Lexer::new(&resolver,"");
        let mut p = Parser::new(&mut lexer).expect("k");
        lexer.import("search:parser/import-smoke").expect("cannot load file");
        let txt = "import-smoke4.dp:1 Reserved keyword \'reserved\' found";
        assert_eq!(txt,p.parse(&mut lexer).ok().unwrap().expect_err("x")[0]);
    }

    #[test]
    fn test_smoke() {
        let config = xxx_test_config();
        let linker = CompilerLink::new(make_compiler_suite().expect("y"));
        let resolver = common_resolver(&config,&linker).expect("a");
        let mut lexer = Lexer::new(&resolver,"");
        let mut p = Parser::new(&mut lexer).expect("k");
        lexer.import("search:parser/parser-smoke").expect("cannot load file");
        p.parse(&mut lexer).expect("parse").map_err(|e| DauphinError::runtime(&e.join(". "))).expect("parse");
        let stmts = p.take_statements();
        let defstore = p.get_defstore();    
        let mut out : Vec<String> = stmts.iter().map(|x| format!("{:?}",x)).collect();
        out.push("".to_string()); /* For trailing \n */
        let outdata = load_testdata(&["parser","parser-smoke.out"]).ok().unwrap();
        assert_eq!(outdata,out.join("\n"));
    }

    #[test]
    fn test_no_nested_dollar() {
        let config = xxx_test_config();
        let linker = CompilerLink::new(make_compiler_suite().expect("y"));
        let resolver = common_resolver(&config,&linker).expect("a");
        let mut lexer = Lexer::new(&resolver,"");
        let mut p = Parser::new(&mut lexer).expect("k");
        lexer.import("search:parser/parser-nonest").expect("cannot load file");
        let txt = "parser-nonest.dp:5 $ encountered outside filter";
        assert_eq!(txt,p.parse(&mut lexer).ok().unwrap().expect_err("x")[0]);
    }

    #[test]
    fn test_id_clash() {
        let config = xxx_test_config();
        let linker = CompilerLink::new(make_compiler_suite().expect("y"));
        let resolver = common_resolver(&config,&linker).expect("a");
        let mut lexer = Lexer::new(&resolver,"");
        let mut p = Parser::new(&mut lexer).expect("k");
        lexer.import("search:parser/id-clash").expect("cannot load file");
        let txt = "id-clash.dp:2 duplicate identifier: id_clash::assign";
        assert_eq!(txt,p.parse(&mut lexer).ok().unwrap().expect_err("x")[0]);
    }

    fn make_identifier(module: &str,name: &str) -> Identifier {
        Identifier::new(module,name)
    }

    fn print_struct(defstore: &DefStore, name: &str) -> String {
        format!("{:?}",defstore.get_struct_id(&make_identifier("struct_smoke",name)).expect("A"))
    }

    #[test]
    fn test_struct() {
        let config = xxx_test_config();
        let linker = CompilerLink::new(make_compiler_suite().expect("y"));
        let resolver = common_resolver(&config,&linker).expect("a");
        let mut lexer = Lexer::new(&resolver,"");
        lexer.import("search:parser/struct-smoke").expect("cannot load file");
        let mut p = Parser::new(&mut lexer).expect("k");
        p.parse(&mut lexer).expect("parse").map_err(|e| DauphinError::runtime(&e.join(". "))).expect("parse");
        let stmts = p.take_statements();
        let defstore = p.get_defstore();
        assert_eq!("struct struct_smoke::A { 0: number, 1: vec(number) }",print_struct(&defstore,"A"));
        assert_eq!("struct struct_smoke::B { X: number, Y: vec(struct_smoke::A) }",print_struct(&defstore,"B"));
        assert_eq!("struct struct_smoke::C {  }",print_struct(&defstore,"C"));
        assert_eq!("[assign(x,A {0: [1,2,3]}), assign(y,B {X: 23,Y: [x,x]})]",&format!("{:?}",stmts));
    }

    fn print_enum(defstore: &DefStore, name: &str) -> String {
        format!("{:?}",defstore.get_enum_id(&make_identifier("enum_smoke",name)).expect("A"))
    }

    #[test]
    fn test_enum() {
        let config = xxx_test_config();
        let linker = CompilerLink::new(make_compiler_suite().expect("y"));
        let resolver = common_resolver(&config,&linker).expect("a");
        let mut lexer = Lexer::new(&resolver,"");
        lexer.import("search:parser/enum-smoke").expect("cannot load file");
        let mut p = Parser::new(&mut lexer).expect("k");
        p.parse(&mut lexer).expect("parse").map_err(|e| DauphinError::runtime(&e.join(". "))).expect("parse");
        let stmts = p.take_statements();
        let defstore = p.get_defstore();    
        assert_eq!("enum enum_smoke::A { M: number, N: vec(number) }",print_enum(&defstore,"A"));
        assert_eq!("enum enum_smoke::B { X: number, Y: vec(enum_smoke::A) }",print_enum(&defstore,"B"));
        assert_eq!("enum enum_smoke::C {  }",print_enum(&defstore,"C"));
        assert_eq!("[assign(x,B:Y [A:M 42,B:N [1,2,3]])]",&format!("{:?}",stmts));
    }

    #[test]
    fn test_short() {
        let config = xxx_test_config();
        let linker = CompilerLink::new(make_compiler_suite().expect("y"));
        let resolver = common_resolver(&config,&linker).expect("a");
        let mut lexer = Lexer::new(&resolver,"");
        lexer.import("search:parser/short").expect("cannot load file");
        let mut p = Parser::new(&mut lexer).expect("k");
        p.parse(&mut lexer).expect("parse").map_err(|e| DauphinError::runtime(&e.join(". "))).expect("parse");
        let stmts = p.take_statements();    
        let modules = stmts.iter().map(|x| (x.0).module().to_string()).collect::<Vec<_>>();
        assert_eq!(vec!["library1","library2","library2",
                        "library1","library2","library1"],modules);
    }

    #[test]
    fn test_macro() {
        let config = xxx_test_config();
        let linker = CompilerLink::new(make_compiler_suite().expect("y"));
        let resolver = common_resolver(&config,&linker).expect("a");
        let mut lexer = Lexer::new(&resolver,"");
        lexer.import("search:parser/macro").expect("cannot load file");
        let mut p = Parser::new(&mut lexer).expect("k");
        p.parse(&mut lexer).expect("parse").map_err(|e| DauphinError::runtime(&e.join(". "))).expect("parse");
        let x = format!("{:?}",p.take_statements());
        print!("{}\n",x);
        assert_eq!("[assign(x,[[1,2,3],[4,5,6],[7,8,9]]), assign(z,0), incr(((x)[eq(@,0)])[eq(@,1)],1), incr(z,plus(0,1)), assign(z,plus(z,0))]",x);
    }

}
