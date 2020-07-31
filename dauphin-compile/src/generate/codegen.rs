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
use super::gencontext::GenContext;
use crate::command::{ Instruction, InstructionType };
use crate::util::DFloat;
use crate::parser::{ Expression, Statement };
use dauphin_interp::command::{ Identifier };
use dauphin_interp::runtime::{ Register };
use dauphin_interp::types::{ BaseType, MemberMode };
use dauphin_interp::util::{ DauphinError, error_locate, triage_source_errors };
use crate::model::DefStore;
use crate::generate::GenerateState;
use crate::typeinf::{ExpressionType, SignatureMemberConstraint, get_constraint };

pub fn add_untyped(context: &mut GenContext, instr: Instruction) -> anyhow::Result<()> {
    let state = context.state_mut();
    let constraint = get_constraint(&instr,&state.defstore()).with_context(|| format!("adding {:?}",instr))?;
    state.typing().add(&constraint)?;
    context.add(instr);
    Ok(())
}

pub fn add_untyped_f(context: &mut GenContext, itype: InstructionType, mut regs_in: Vec<Register>) -> anyhow::Result<Register> {
    let state = context.state_mut();
    let dst = state.regalloc().allocate();
    let mut regs = vec![dst];
    regs.append(&mut regs_in);
    let instr = Instruction::new(itype,regs);
    add_untyped(context,instr)?;
    Ok(dst)
}

macro_rules! addf {
    ($this:expr,$opcode:tt,$($regs:expr),*) => {
        add_untyped_f(&mut $this.context,InstructionType::$opcode,vec![$($regs),*])?
    };
    ($this:expr,$opcode:tt($($args:expr),*),$($regs:expr),*) => {
        add_untyped_f(&mut $this.context,InstructionType::$opcode($($args),*),vec![$($regs),*])?
    };
    ($this:expr,$opcode:tt) => {
        add_untyped_f(&mut $this.context,InstructionType::$opcode,vec![])?
    };
    ($this:expr,$opcode:tt($($args:expr),*)) => {
        add_untyped_f(&mut $this.context,InstructionType::$opcode($($args),*),vec![])?
    };
}

pub struct CodeGen<'b> {
    context: GenContext<'b>,
    include_line_numbers: bool
}

impl<'b> CodeGen<'b> {
    fn new(state: &'b mut GenerateState, include_line_numbers: bool) -> CodeGen<'b> {
        CodeGen {
            context: GenContext::new(state),
            include_line_numbers
        }
    }

    fn build_vec(&mut self, values: &Vec<Expression>, dollar: Option<&Register>, at: Option<&Register>) -> anyhow::Result<Register> {
        let tmp = addf!(self,Nil);
        for val in values {
            let r = self.build_rvalue(val,dollar,at)?;
            add_untyped(&mut self.context,Instruction::new(InstructionType::Append,vec![tmp,r]))?;
        }
        Ok(addf!(self,Star,tmp))

    }

    fn struct_rearrange(&mut self, s: &Identifier, x: Vec<Register>, got_names: &Vec<String>) -> anyhow::Result<Vec<Register>> {
        let decl = self.context.state().defstore().get_struct_id(s)?;
        let gotpos : HashMap<String,usize> = got_names.iter().enumerate().map(|(i,e)| (e.to_string(),i)).collect();
        let mut out = Vec::new();
        for want_name in decl.get_names().iter() {
            if let Some(got_pos) = gotpos.get(want_name) {
                out.push(x[*got_pos]);
            } else {
                return Err(DauphinError::source(&format!("Missing member '{}'",want_name)));
            }
        }
        Ok(out)
    }

    fn type_of(&mut self, expr: &Expression) -> anyhow::Result<ExpressionType> {
        Ok(match expr {
            Expression::Identifier(id) => {
                let reg = self.context.state_mut().codegen_regnames().lookup_input(id)?.clone();
                let state = self.context.state_mut();
                state.typing().get(&reg)
            },
            Expression::Dot(x,f) => {
                if let ExpressionType::Base(BaseType::StructType(name)) = self.type_of(x)? {
                    let struct_ = self.context.state().defstore().get_struct_id(&name)?;
                    if let Some(type_) = struct_.get_member_type(f) {
                        type_.to_expressiontype()
                    } else {
                        return Err(DauphinError::source(&format!("no such field {:?}",f)));
                    }
                } else {
                    return Err(DauphinError::source(&format!("{:?} is not a structure",expr)));
                }
            },
            Expression::Pling(x,f) => {
                if let ExpressionType::Base(BaseType::EnumType(name)) = self.type_of(x)? {
                    let enum_ = self.context.state().defstore().get_enum_id(&name)?;
                    if let Some(type_) = enum_.get_branch_type(f) {
                        type_.to_expressiontype()
                    } else {
                        return Err(DauphinError::source(&format!("no such field {:?}",f)));
                    }
                } else {
                    return Err(DauphinError::source(&format!("{:?} is not a structure",expr)));
                }
            },
            Expression::Square(x) | Expression::Bracket(x,_) => {
                if let ExpressionType::Vec(subtype) = self.type_of(x)? {
                    subtype.as_ref().clone()
                } else {
                    return Err(DauphinError::source(&format!("{:?} is not a vector",expr)));
                }
            },
            Expression::Filter(x,_) => {
                self.type_of(x)?
            },
            _ => return Err(DauphinError::source(&format!("Cannot type {:?}",expr)))
        })
    }

    fn build_lvalue(&mut self, expr: &Expression, stomp: bool, unfiltered_in: bool) -> anyhow::Result<(Register,Option<Register>,Register)> {
        match expr {
            Expression::Identifier(id) => {
                let state = self.context.state_mut();
                let alloc = state.regalloc().clone();
                let real_reg = self.context.state_mut().codegen_regnames().lookup_output(id,stomp,&alloc)?;
                let lvalue_reg = addf!(self,Alias,real_reg);
                Ok((lvalue_reg,None,real_reg))
            },
            Expression::Dot(x,f) => {
                if let ExpressionType::Base(BaseType::StructType(name)) = self.type_of(x)? {
                    let (lvalue_subreg,fvalue_reg,rvalue_subreg) = self.build_lvalue(x,false,unfiltered_in)?;
                    let lvalue_reg = addf!(self,RefSValue(name.clone(),f.clone()),lvalue_subreg);
                    let rvalue_reg = addf!(self,SValue(name.clone(),f.clone()),rvalue_subreg);
                    Ok((lvalue_reg,fvalue_reg,rvalue_reg))
                } else {
                    Err(DauphinError::source("Can only take \"dot\" of structs"))
                }
            },
            Expression::Pling(x,f) => {
                if let ExpressionType::Base(BaseType::EnumType(name)) = self.type_of(x)? {
                    let (lvalue_subreg,fvalue_subreg,rvalue_subreg) = self.build_lvalue(x,false,unfiltered_in)?;
                    let lvalue_reg = addf!(self,RefEValue(name.clone(),f.clone()),lvalue_subreg);
                    let mut fvalue_reg = addf!(self,FilterEValue(name.clone(),f.clone()),rvalue_subreg);
                    if let Some(fvalue_subreg) = fvalue_subreg {
                        fvalue_reg = addf!(self,ReFilter,fvalue_subreg,fvalue_reg);
                    }
                    let rvalue_reg = addf!(self,EValue(name.clone(),f.clone()),rvalue_subreg);
                    Ok((lvalue_reg,Some(fvalue_reg),rvalue_reg))
                } else {
                    Err(DauphinError::source("Can only take \"pling\" of enums"))
                }
            },
            Expression::Square(x) => {
                let (lvalue_subreg,_,rvalue_subreg) = self.build_lvalue(x,false,false)?;
                let lvalue_reg = addf!(self,RefSquare,lvalue_subreg);
                let rvalue_reg = addf!(self,Square,rvalue_subreg);
                let fvalue_reg = addf!(self,FilterSquare,rvalue_subreg);
                Ok((lvalue_reg,Some(fvalue_reg),rvalue_reg))
            },
            Expression::Filter(x,f) => {
                let (lvalue_reg,fvalue_subreg,rvalue_subreg) = self.build_lvalue(x,false,false)?;
                /* Unlike in a bracket, @ makes no sense in a filter as the array has already been lost */
                let filterreg = self.build_rvalue(f,Some(&rvalue_subreg),None)?;
                let fvalue_reg = if let Some(fvalue_subreg) = fvalue_subreg {
                    addf!(self,Filter,fvalue_subreg,filterreg)
                } else {
                    let atreg = addf!(self,At,rvalue_subreg);
                    addf!(self,Filter,atreg,filterreg)
                };
                let rvalue_reg = addf!(self,Filter,rvalue_subreg,filterreg);
                Ok((lvalue_reg,Some(fvalue_reg),rvalue_reg))
            },
            Expression::Bracket(x,f) => {
                let (lvalue_subreg,_,rvalue_subreg) = self.build_lvalue(x,false,false)?;
                let lvalue_reg = addf!(self,RefSquare,lvalue_subreg);
                let rvalue_interreg = addf!(self,Square,rvalue_subreg);
                let fvalue_interreg = addf!(self,FilterSquare,rvalue_subreg);
                let atreg = addf!(self,At,rvalue_subreg);
                let filterreg = self.build_rvalue(f,Some(&rvalue_interreg),Some(&atreg))?;
                let fvalue_reg = addf!(self,Filter,fvalue_interreg,filterreg);
                let rvalue_reg = addf!(self,Filter,rvalue_interreg,filterreg);
                Ok((lvalue_reg,Some(fvalue_reg),rvalue_reg))
            },
            _ => return Err(DauphinError::source(&"Invalid lvalue".to_string()))
        }
    }

    fn build_rvalue(&mut self, expr: &Expression, dollar: Option<&Register>, at: Option<&Register>) -> anyhow::Result<Register> {
        Ok(match expr {
            Expression::Identifier(id) => {
                self.context.state_mut().codegen_regnames().lookup_input(id)?.clone()
            },
            Expression::Number(n) =>        
                addf!(self,NumberConst(DFloat::new_str(n).map_err(|_| DauphinError::source(&format!("Bad number '{:?}'",n)))?)),
            Expression::LiteralString(s) => addf!(self,StringConst(s.to_string())),
            Expression::LiteralBool(b) =>   addf!(self,BooleanConst(*b)),
            Expression::LiteralBytes(b) =>  addf!(self,BytesConst(b.to_vec())),
            Expression::Vector(v) =>        self.build_vec(v,dollar,at)?,
            Expression::Operator(identifier,x) => {
                let mut subregs = vec![];
                for e in x {
                    let r = self.build_rvalue(e,dollar,at)?;
                    subregs.push(r);
                }
                add_untyped_f(&mut self.context, InstructionType::Operator(identifier.clone()),subregs)?
            },
            Expression::CtorStruct(s,x,n) => {
                let mut subregs = vec![];
                for e in x {
                    let r = self.build_rvalue(e,dollar,at)?;
                    subregs.push(r);
                }
                let out = self.struct_rearrange(s,subregs,n)?;
                add_untyped_f(&mut self.context, InstructionType::CtorStruct(s.clone()),out)?
            },
            Expression::CtorEnum(e,b,x) => {
                let subreg = self.build_rvalue(x,dollar,at)?;
                addf!(self,CtorEnum(e.clone(),b.clone()),subreg)
            },
            Expression::Dot(x,f) => {
                let subreg = self.build_rvalue(x,dollar,at)?;
                let state = self.context.state_mut();
                let stype = state.typing().get(&subreg);
                if let ExpressionType::Base(BaseType::StructType(name)) = stype {
                    addf!(self,SValue(name.clone(),f.clone()),subreg)
                } else {
                    return Err(DauphinError::source(&format!("Can only take \"dot\" of structs, not {:?}",stype)));
                }
            },
            Expression::Query(x,f) => {
                let subreg = self.build_rvalue(x,dollar,at)?;
                let state = self.context.state_mut();
                let etype = state.typing().get(&subreg);
                if let ExpressionType::Base(BaseType::EnumType(name)) = etype {
                    addf!(self,ETest(name.clone(),f.clone()),subreg)
                } else {
                    return Err(DauphinError::source("Can only take \"query\" of enums"));
                }
            },
            Expression::Pling(x,f) => {
                let subreg = self.build_rvalue(x,dollar,at)?;
                let state = self.context.state_mut();
                let etype = state.typing().get(&subreg);
                if let ExpressionType::Base(BaseType::EnumType(name)) = etype {
                    addf!(self,EValue(name.clone(),f.clone()),subreg)
                } else {
                    return Err(DauphinError::source("Can only take \"pling\" of enums"));
                }
            },
            Expression::Square(x) => {
                let subreg = self.build_rvalue(x,dollar,at)?;
                addf!(self,Square,subreg)
            },
            Expression::Star(x) => {
                let subreg = self.build_rvalue(x,dollar,at)?;
                addf!(self,Star,subreg)
            },
            Expression::Filter(x,f) => {
                let subreg = self.build_rvalue(x,dollar,at)?;
                /* Unlike in a bracket, @ makes no sense in a filter as the array has already been lost */
                let filterreg = self.build_rvalue(f,Some(&subreg),None)?;
                addf!(self,Filter,subreg,filterreg)
            },
            Expression::Bracket(x,f) => {
                let subreg = self.build_rvalue(x,dollar,at)?;
                let atreg = addf!(self,At,subreg);
                let sq_subreg = addf!(self,Square,subreg);
                let filterreg = self.build_rvalue(f,Some(&sq_subreg),Some(&atreg))?;
                addf!(self,Filter,sq_subreg,filterreg)
            },
            Expression::Dollar => {
                if let Some(dollar) = dollar {
                    addf!(self,Copy,*dollar)
                } else {
                    return Err(DauphinError::source("Unexpected $"));
                }
            },
            Expression::At => {
                if let Some(at) = at {
                    addf!(self,Copy,*at)
                } else {
                    return Err(DauphinError::source("Unexpected @"));
                }
            }
        })
    }

    fn build_stmt(&mut self, stmt: &Statement) -> anyhow::Result<()> {
        let mut regs = Vec::new();
        let mut modes = Vec::new();
        if self.include_line_numbers {
            add_untyped(&mut self.context,Instruction::new(InstructionType::LineNumber(stmt.2.clone()),vec![]))?;    
        }
        let sig = self.context.state().defstore().get_proc_id(&stmt.0)?.get_signature().clone();
        for (i,member) in sig.each_member().enumerate() {
            match member {
                SignatureMemberConstraint::RValue(_) => {
                    modes.push(MemberMode::In);
                    regs.push(self.build_rvalue(&stmt.1[i],None,None)?);
                },
                SignatureMemberConstraint::LValue(_,stomp) => {
                    let (lvalue_reg,fvalue_reg,_) = self.build_lvalue(&stmt.1[i],*stomp,true)?;
                    if let Some(fvalue_reg) = fvalue_reg {
                        modes.push(MemberMode::Filter);
                        regs.push(fvalue_reg);
                    }
                    modes.push(if fvalue_reg.is_none() && *stomp { MemberMode::Out } else { MemberMode::InOut });
                    regs.push(lvalue_reg);
                }
            }
        }
        add_untyped(&mut self.context,Instruction::new(InstructionType::Proc(stmt.0.clone(),modes),regs))?;
        self.context.state_mut().codegen_regnames().commit();
        Ok(())
    }

    fn go(mut self, stmts: &[Statement]) -> anyhow::Result<Result<GenContext<'b>,Vec<String>>> {
        let mut errors = vec![];
        for stmt in stmts {
            if let Err(e) = self.build_stmt(stmt) {
                let pos = &stmt.2;
                errors.push(error_locate(e,pos.filename(),pos.line()));
            }
        }
        if errors.len() > 0 {
            match triage_source_errors(&mut errors) {
                Ok(e) => Ok(Err(e)),
                Err(e) => Err(e)
            }
        } else {
            let state = self.context.state_mut();
            for (reg,expression_type) in state.typing().all_external() {
                state.types().set(&reg,&expression_type.to_membertype(&BaseType::BooleanType));
            }
            Ok(Ok(self.context))
        }
    }
}

pub fn generate_code<'b>(state: &'b mut GenerateState, stmts: &Vec<Statement>, include_line_numbers: bool) -> anyhow::Result<Result<GenContext<'b>,Vec<String>>> {
    let mut out = CodeGen::new(state,include_line_numbers).go(stmts)?;
    if let Ok(ref mut context) = out {
        context.phase_finished();
    }
    Ok(out)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lexer::Lexer;
    use crate::resolver::common_resolver;
    use crate::command::CompilerLink;
    use crate::parser::Parser;
    use crate::test::{ xxx_test_config, make_compiler_suite };
    use dauphin_test_harness::load_testdata;

    fn run_pass(filename: &str) -> Result<(),Vec<String>> {
        let config = xxx_test_config();
        let linker = CompilerLink::new(make_compiler_suite().expect("y"));
        let resolver = common_resolver(&config,&linker).expect("a");
        let mut lexer = Lexer::new(&resolver,"");
        lexer.import(&format!("search:codegen/{}",filename)).expect("cannot load file");
        let mut state = GenerateState::new("test");
        let mut p = Parser::new(&mut state,&mut lexer).expect("k");
        p.parse(&mut state,&mut lexer).expect("error").expect("error");
        let stmts = p.take_statements();
        let gen = CodeGen::new(&mut state,true);
        gen.go(&stmts).expect("go")?;
        Ok(())
    }

    #[test]
    fn codegen_smoke() {
        let config = xxx_test_config();
        let linker = CompilerLink::new(make_compiler_suite().expect("y"));
        let resolver = common_resolver(&config,&linker).expect("a");
        let mut lexer = Lexer::new(&resolver,"");
        lexer.import("search:codegen/generate-smoke2").expect("cannot load file");
        let mut state = GenerateState::new("test");
        let mut p = Parser::new(&mut state,&mut lexer).expect("k");
        p.parse(&mut state,&mut lexer).expect("error").expect("error");
        let stmts = p.take_statements();
        let gencontext = generate_code(&mut state,&stmts,true).expect("codegen").expect("error");
        let cmds : Vec<String> = gencontext.get_instructions().iter().map(|e| format!("{:?}",e)).collect();
        let outdata = load_testdata(&["codegen","generate-smoke2.out"]).ok().unwrap();
        print!("{}",cmds.join(""));
        assert_eq!(outdata,cmds.join(""));
    }

    #[test]
    fn codegen_lvalue_checks() {
        run_pass("typepass-reassignok").expect("A");
        run_pass("typepass-reassignbad").expect_err("B");
    }
}
