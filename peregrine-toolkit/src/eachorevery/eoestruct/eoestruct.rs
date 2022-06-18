use std::{sync::Arc };
use crate::eachorevery::{EachOrEvery, EachOrEveryGroupCompatible};
use identitynumber::{ identitynumber };
use lazy_static::lazy_static;

#[cfg(debug_assertions)]
use super::{eoestructformat::{VariableSystemFormatter}};

/* EoeStructs use a number of different tree types during processing:
 *   TemplateTree -- defined by the user and includes EoE arrays. Composable.
 *   BuilderTree -- built from a template tree and ready to expand. Read-only.
 *   NullTree -- an output tree.
 * 
 * This file contains definitions not specific to any of the particular tree types.
 * StructValueId -- a singleton allowing matching of variable decls and use.
 * StructConst -- a primitive atomic value (number/string/boolean/null).
 * StructVarvalue -- an EoE of a primitive atomic value (number/string/boolean/null).
 * StructPair -- key value pair for objects.
 * VariableSystem -- a trait declaring the types of the Var and All nodes for any given tree type.
 * Struct -- a tree
 * StructVisitor -- a visitor trait for a Struct
 */

identitynumber!(IDS);

#[cfg(debug_assertions)]
pub type StructError = String;

#[cfg(debug_assertions)]
pub(super) fn struct_error(msg: &str) -> StructError { msg.to_string() }

#[cfg(not(debug_assertions))]
pub type StructError = ();

#[cfg(not(debug_assertions))]
pub(super) fn struct_error(msg: &str) -> StructError { () }

pub type StructResult = Result<(),StructError>;

#[derive(Copy,Clone,PartialEq,Eq,Hash)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct StructValueId(pub(super) u64);

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

#[derive(Clone)]
/* Guarantee: all EachOrEverys in here will be Each after construction */
pub enum StructVarValue {
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
        let mut sep = false;
        write!(f,"<")?;
        for value in value.iter(len).unwrap() {
            if sep { write!(f,",")?; }
            write!(f,"{:?}",value)?;
            sep = true;
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

pub struct StructPair<T: VariableSystem+Clone>(pub String,pub Struct<T>);

pub trait VariableSystem {
    type Declare;
    type Use;

    #[cfg(debug_assertions)]
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
    fn visit_const(&mut self, _input: &StructConst) -> StructResult { Ok(()) }
    fn visit_var(&mut self, _input: &T::Use) -> StructResult { Ok(()) }
    fn visit_array_start(&mut self) -> StructResult { Ok(()) }
    fn visit_array_end(&mut self) -> StructResult { Ok(()) }
    fn visit_object_start(&mut self) -> StructResult { Ok(()) }
    fn visit_object_end(&mut self) -> StructResult { Ok(()) }
    fn visit_pair_start(&mut self, _key: &str) -> StructResult { Ok(()) }
    fn visit_pair_end(&mut self, _key: &str) -> StructResult { Ok(()) }
    fn visit_all_start(&mut self, _id: &[T::Declare]) -> StructResult { Ok(()) }
    fn visit_all_end(&mut self, _id: &[T::Declare]) -> StructResult { Ok(()) }
}

impl<T: Clone+VariableSystem> Struct<T> {
    pub(super) fn visit(&self, visitor: &mut dyn StructVisitor<T>) -> StructResult {
        match self {
            Struct::Const(input) => visitor.visit_const(input)?,
            Struct::Var(input) => visitor.visit_var(input)?,
            Struct::Array(input) => {
                visitor.visit_array_start()?;
                for value in input.iter() {
                    value.visit(visitor)?;
                }
                visitor.visit_array_end()?;
            },
            Struct::Object(input) => {
                visitor.visit_object_start()?;
                for value in input.iter() {
                    visitor.visit_pair_start(&value.0)?;
                    value.1.visit(visitor)?;
                    visitor.visit_pair_end(&value.0)?;
                }
                visitor.visit_object_end()?;

            },
            Struct::All(vars, expr) => {
                visitor.visit_all_start(vars)?;
                expr.visit(visitor)?;
                visitor.visit_all_end(vars)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::{eachorevery::eoestruct::{eoejson::{struct_to_json, struct_from_json}}};
    use serde_json::{Value as JsonValue, Number};

    fn json_fix_numbers(json: &JsonValue) -> JsonValue {
        match json {
            JsonValue::Null => JsonValue::Null,
            JsonValue::Bool(x) => JsonValue::Bool(*x),
            JsonValue::Number(n) => JsonValue::Number(Number::from_f64(n.as_f64().unwrap()).unwrap()),
            JsonValue::String(s) => JsonValue::String(s.to_string()),
            JsonValue::Array(x) => JsonValue::Array(x.iter().map(|x| json_fix_numbers(x)).collect()),
            JsonValue::Object(x) => JsonValue::Object(x.iter().map(|(k,v)| (k.to_string(),json_fix_numbers(v))).collect()),
        }
    }

    macro_rules! json_get {
        ($name:ident,$var:tt,$typ:ty) => {
            fn $name(value: &JsonValue) -> $typ {
                match value {
                    JsonValue::$var(v) => v.clone(),
                    _ => panic!("malformatted test data")
                }
            }
                    
        };
    }

    json_get!(json_array,Array,Vec<JsonValue>);
    json_get!(json_string,String,String);

    fn run_case(value: &JsonValue) {
        let parts = json_array(value);
        println!("ruuning {}\n",json_string(&parts[0]));
        let vars = json_array(&parts[1]).iter().map(|x| json_string(x)).collect::<Vec<_>>();
        let template = struct_from_json(vars,&parts[2]).ok().unwrap();
        let debug = format!("{:?}",template);
        let output = struct_to_json(template.build().ok().expect("unexpected error")).ok().unwrap();
        let output = JsonValue::from_str(&output.to_string()).ok().unwrap();
        assert_eq!(debug,json_string(&parts[3]));
        assert_eq!(json_fix_numbers(&output),json_fix_numbers(&parts[4]));
        println!("{:?}\n",template);
        println!("{:?}\n",json_fix_numbers(&output));
    }

    fn run_case_buildfail(value: &JsonValue) {
        let parts = json_array(value);
        println!("ruuning {}\n",json_string(&parts[0]));
        let vars = json_array(&parts[1]).iter().map(|x| json_string(x)).collect::<Vec<_>>();
        let template = struct_from_json(vars,&parts[2]).ok().unwrap();
        match template.build() {
            Ok(r) => { eprintln!("unexpected success: {:?}",r); assert!(false); },
            Err(e) => assert_eq!(e,json_string(&parts[3]))
        }
    }

    fn run_case_parsefail(value: &JsonValue) {
        let parts = json_array(value);
        println!("ruuning {}\n",json_string(&parts[0]));
        let vars = json_array(&parts[1]).iter().map(|x| json_string(x)).collect::<Vec<_>>();
        match struct_from_json(vars,&parts[2]) {
            Ok(r) => { eprintln!("unexpected success: {:?}",r); assert!(false); },
            Err(e) => assert_eq!(e,json_string(&parts[3]))
        }
    }

    #[test]
    fn test_eoestruct_smoke() {
        let data = JsonValue::from_str(include_str!("test-eoe-smoke.json")).ok().unwrap();
        for testcase in json_array(&data).iter() {
            run_case(&testcase);
        }
    } 

    #[test]
    fn test_eoestruct_buildfail() {
        let data = JsonValue::from_str(include_str!("test-eoe-buildfail.json")).ok().unwrap();
        for testcase in json_array(&data).iter() {
            run_case_buildfail(&testcase);
        }
    } 

    #[test]
    fn test_eoestruct_parsefail() {
        let data = JsonValue::from_str(include_str!("test-eoe-parsefail.json")).ok().unwrap();
        for testcase in json_array(&data).iter() {
            run_case_parsefail(&testcase);
        }
    } 
}