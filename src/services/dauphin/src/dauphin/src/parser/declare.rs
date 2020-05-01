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

use super::node::{ ParserStatement, ParseError };
use super::lexutil::not_reserved;
use crate::model::{
    InlineMode, Inline, DefStore, ExprMacro, StmtMacro, FuncDecl, ProcDecl,
    StructDef, EnumDef
};
use crate::lexer::Lexer;
use crate::typeinf::{ SignatureConstraint, MemberType };

fn run_import(path: &str, lexer: &mut Lexer) -> Result<(),ParseError> {
    lexer.import(path).map_err(|s| ParseError::new(&format!("import failed: {}",s),lexer))
}

fn run_inline(symbol: &str, name: &str, mode: &InlineMode, prio: f64, lexer: &mut Lexer, defstore: &mut DefStore) -> Result<(),ParseError> {
    let stmt_like = defstore.stmt_like(None,name,lexer)?; /// XXX module
    lexer.add_inline(symbol,mode == &InlineMode::Prefix).map_err(|s| {
        ParseError::new(&s,lexer)
    })?;
    defstore.add_inline(Inline::new(symbol,name,stmt_like,prio,mode))?;
    Ok(())
}

fn run_expr(name: &str, defstore: &mut DefStore, lexer: &mut Lexer) -> Result<(),ParseError> {
    not_reserved(name,lexer)?;
    defstore.add_expr(ExprMacro::new(name),lexer)?;
    Ok(())
}

fn run_stmt(name: &str, defstore: &mut DefStore, lexer: &mut Lexer) -> Result<(),ParseError> {
    not_reserved(name,lexer)?;
    defstore.add_stmt(StmtMacro::new(name),lexer)?;
    Ok(())
}

fn run_proc(name: &str, signature: &SignatureConstraint, defstore: &mut DefStore, lexer: &mut Lexer) -> Result<(),ParseError> {
    not_reserved(name,lexer)?;
    defstore.add_proc(ProcDecl::new("",name,signature),lexer)?; // XXX module name
    Ok(())
}

fn run_func(name: &str, signature: &SignatureConstraint, defstore: &mut DefStore, lexer: &mut Lexer) -> Result<(),ParseError> {
    not_reserved(name,lexer)?;
    defstore.add_func(FuncDecl::new("",name,signature),lexer)?;  // XXX module name
    Ok(())
}

fn run_struct(name: &str, member_types: &Vec<MemberType>, names: &Vec<String>, defstore: &mut DefStore, lexer: &mut Lexer) -> Result<(),ParseError> {
    not_reserved(name,lexer)?;
    let def = StructDef::new(name,member_types,names).map_err(|e| ParseError::new(&e,lexer) )?;
    defstore.add_struct(def,lexer)?;
    Ok(())
}

// TODO allow one operator as prefix of another
fn run_enum(name: &str, member_types: &Vec<MemberType>, names: &Vec<String>, defstore: &mut DefStore, lexer: &mut Lexer) -> Result<(),ParseError> {
    not_reserved(name,lexer)?;
    let def = EnumDef::new(name,member_types,names).map_err(|e| ParseError::new(&e,lexer) )?;
    defstore.add_enum(def,lexer)?;
    Ok(())
}

pub fn declare(stmt: &ParserStatement, lexer: &mut Lexer, defstore: &mut DefStore) -> Result<bool,ParseError> {
    match stmt {
        ParserStatement::Import(path) =>
            run_import(path,lexer).map(|_| true),
        ParserStatement::Inline(symbol,name,mode,prio) => 
            run_inline(&symbol,&name,mode,*prio,lexer,defstore).map(|_| true),
        ParserStatement::ExprMacro(name) =>
            run_expr(&name,defstore,lexer).map(|_| true),
        ParserStatement::StmtMacro(name) =>
            run_stmt(&name,defstore,lexer).map(|_| true),
        ParserStatement::ProcDecl(name,signature) =>
            run_proc(&name,&signature,defstore,lexer).map(|_| true),
        ParserStatement::FuncDecl(name,signature) =>
            run_func(&name,signature,defstore,lexer).map(|_| true),
        ParserStatement::StructDef(name,member_types,names) =>
            run_struct(&name,&member_types,&names,defstore,lexer).map(|_| true),
        ParserStatement::EnumDef(name,member_types,names) =>
            run_enum(&name,&member_types,names,defstore,lexer).map(|_| true),
        _ => { return Ok(false); }
    }
}
