use std::{sync::Arc, collections::HashMap };
use crate::eachorevery::{EachOrEvery, EachOrEveryGroupCompatible};

use super::{eoestructformat::{VariableSystemFormatter, StructDebug}, buildertree::{BuiltVars, TemplateBuildVisitor}};
use identitynumber::{ identitynumber };
use lazy_static::lazy_static;

identitynumber!(IDS);

#[derive(Copy,Clone,PartialEq,Eq,Hash)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub(super) struct StructValueId(pub(super) u64);

impl StructValueId {
    pub(super) fn new() -> StructValueId { StructValueId(IDS.next()) }
}

#[derive(Clone)]
pub enum StructConst {
    Number(f64),
    String(String),
    Boolean(bool),
    Null
}

pub trait StructConstVisitor {
    fn visit_number(&mut self, value: f64);
    fn visit_string(&mut self, value: &str);
    fn visit_boolean(&mut self, value: bool);
    fn visit_null(&mut self);
}

impl StructConst {
    pub(super) fn visit(&self, visitor: &mut dyn StructConstVisitor) {
        match self {
            StructConst::Number(input) => visitor.visit_number(*input),
            StructConst::String(input) => visitor.visit_string(input),
            StructConst::Boolean(input) => visitor.visit_boolean(*input),
            StructConst::Null => visitor.visit_null()
        }
    }
}

#[derive(Clone)]
/* Guarantee: all EachOrEverys in here will be Each after construction */
pub(super) enum StructVarValue {
    Number(EachOrEvery<f64>),
    String(EachOrEvery<String>),
    Boolean(EachOrEvery<bool>),
}

fn to_const<X>(input: &EachOrEvery<X>) -> Option<&X> {
    if input.len().is_none() {
        Some(input.get(0).unwrap())
    } else {
        None
    }
}
 
fn format<X: std::fmt::Debug>(value: &EachOrEvery<X>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if let Some(len) = value.len() {
        let mut sep = "<";
        for value in value.iter(len).unwrap() {
            write!(f,"{}{:?}",sep,value)?;
            sep = ",";
        }
        write!(f,">")?;
    } else {
        let value = value.iter(1).unwrap().next().unwrap();
        write!(f,"{:?}",value)?;
    }
    Ok(())
}

impl std::fmt::Debug for StructVarValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StructVarValue::Number(x) => format(x,f),
            StructVarValue::String(x) => format(x,f),
            StructVarValue::Boolean(x) => format(x,f)
        }
    }
}

impl StructVarValue {
    pub(super) fn to_const(&self) -> Option<StructConst> {
        match self {
            StructVarValue::Number(input) => {
                to_const(input).map(|x| StructConst::Number(*x))
            },
            StructVarValue::String(input) => {
                to_const(input).map(|x| StructConst::String(x.clone()))
            },
            StructVarValue::Boolean(input) => {
                to_const(input).map(|x| StructConst::Boolean(*x))
            }
        }
    }

    pub(super) fn check_compatible(&self, compat: &mut EachOrEveryGroupCompatible) {
        match self {
            StructVarValue::Number(input) => compat.add(input),
            StructVarValue::String(input) => compat.add(input),
            StructVarValue::Boolean(input) => compat.add(input)
        };
    }

    pub(super) fn get<'a>(&'a self, index: usize) -> Option<StructConst> {
        match self {
            StructVarValue::Number(input) => {
                input.get(index).map(|x| StructConst::Number(*x))
            },
            StructVarValue::String(input) => {
                input.get(index).map(|x| StructConst::String(x.to_string()))
            },
            StructVarValue::Boolean(input) => {
                input.get(index).map(|x| StructConst::Boolean(*x))
            }
        }
    }
}

pub struct StructPair<T: VariableSystem+Clone>(pub(super) String,pub(super) Struct<T>);

pub trait VariableSystem {
    type Declare;
    type Use;

    fn build_formatter() -> Box<dyn VariableSystemFormatter<Self>>;
}

#[derive(Clone)]
pub enum Struct<T: VariableSystem+Clone> {
    Var(T::Use),
    Const(StructConst),
    Array(Arc<Vec<Struct<T>>>),
    Object(Arc<Vec<StructPair<T>>>),
    All(Vec<T::Declare>,Arc<Struct<T>>)
}

pub(super) trait StructVisitor<T: VariableSystem+Clone> {
    fn visit_const(&mut self, input: &StructConst) {}
    fn visit_var(&mut self, input: &T::Use) {}
    fn visit_array_start(&mut self) {}
    fn visit_array_end(&mut self) {}
    fn visit_object_start(&mut self) {}
    fn visit_object_end(&mut self) {}
    fn visit_pair_start(&mut self, key: &str) {}
    fn visit_pair_end(&mut self, key: &str) {}
    fn visit_all_start(&mut self, id: &[T::Declare]) {}
    fn visit_all_end(&mut self, id: &[T::Declare]) {}
}

impl<T: Clone+VariableSystem> Struct<T> {
    pub(super) fn visit(&self, visitor: &mut dyn StructVisitor<T>) {
        match self {
            Struct::Const(input) => visitor.visit_const(input),
            Struct::Var(input) => visitor.visit_var(input),
            Struct::Array(input) => {
                visitor.visit_array_start();
                for value in input.iter() {
                    value.visit(visitor);
                }
                visitor.visit_array_end();
            },
            Struct::Object(input) => {
                visitor.visit_object_start();
                for value in input.iter() {
                    visitor.visit_pair_start(&value.0);
                    value.1.visit(visitor);
                    visitor.visit_pair_end(&value.0);
                }
                visitor.visit_object_end();

            },
            Struct::All(vars, expr) => {
                visitor.visit_all_start(vars);
                expr.visit(visitor);
                visitor.visit_all_end(vars);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::eachorevery::eoestruct::templatetree::StructVar;

    use super::*;

    #[test]
    fn test_eoestruct_smoke() {
        // [ 1,2, Aa: ( Ab: ( [a=<10,11>,b=<20,21,22>,30] )) ]
        let a = StructVar::new_number(EachOrEvery::each(vec![10.,11.]));
        let b = StructVar::new_number(EachOrEvery::each(vec![20.,21.,22.]));
        let c = StructVar::new_number(EachOrEvery::every(30.));
        let abc = Struct::new_array(vec![
            Struct::new_var(a.clone()),
            Struct::new_var(b.clone()),
            Struct::new_var(c)
        ]);
        let all = Struct::new_all(&[a],Struct::new_all(&[b],abc));
        let out1 = StructVar::new_number(EachOrEvery::every(1.));
        let out2 = Struct::new_number(2.);
        let outer = Struct::new_array(vec![
            Struct::new_var(out1),
            out2,
            all
        ]);
        println!("{:?}",outer);
        assert_eq!("[1.0,2.0,Aa.( Ab.( [a=<10.0,11.0>,b=<20.0,21.0,22.0>,30.0] ) )]",&format!("{:?}",outer));
        let splitter = outer.splitter();
        println!("{:?}",splitter);
        let mut expander = StructDebug::new();
        splitter.expand(&mut expander.visitor());
        println!("{}",expander.out());
        // [ 1,2, Aa,b: ( { "a:" a=<10,11,12>, "b": b=<20,21,22>, "c": 30 } ) ]
        let a = StructVar::new_number(EachOrEvery::each(vec![10.,11.,12.]));
        let b = StructVar::new_number(EachOrEvery::each(vec![20.,21.,22.]));
        let c = StructVar::new_number(EachOrEvery::every(30.));
        let abc = Struct::new_object(vec![
            StructPair::new("a",Struct::new_var(a.clone())),
            StructPair::new("b",Struct::new_var(b.clone())),
            StructPair::new("c",Struct::new_var(c)),
        ]);
        let all = Struct::new_all(&[a,b],abc);
        let out1 = StructVar::new_number(EachOrEvery::every(1.));
        let out2 = Struct::new_number(2.);
        let outer = Struct::new_array(vec![
            Struct::new_var(out1),
            out2,
            all
        ]);
        println!("{:?}",outer);
        assert_eq!("[1.0,2.0,Aab.( {\"a\": a=<10.0,11.0,12.0>,\"b\": b=<20.0,21.0,22.0>,\"c\": 30.0} )]",&format!("{:?}",outer));
        let splitter = outer.splitter();
        println!("{:?}",splitter);
        let mut expander = StructDebug::new();
        splitter.expand(&mut expander.visitor());
        println!("{}",expander.out());
    } 
}
