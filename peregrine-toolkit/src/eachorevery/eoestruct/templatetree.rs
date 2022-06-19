use std::sync::Arc;
use crate::eachorevery::EachOrEvery;
use super::{eoestruct::{StructConst, StructValueId, StructVarValue}};

#[derive(Clone)]
pub struct StructVar {
    pub(super) value: StructVarValue,
    pub(super) id: StructValueId
}

impl StructVar {
    fn to_const(&self) -> Option<StructConst> { self.value.to_const() }

    fn new(value: StructVarValue) -> StructVar {
        StructVar { value, id: StructValueId::new() }
    }

    pub fn new_number(input: EachOrEvery<f64>) -> StructVar {
        Self::new(StructVarValue::Number(input))
    }

    pub fn new_string(input: EachOrEvery<String>) -> StructVar {
        Self::new(StructVarValue::String(input))
    }

    pub fn new_boolean(input: EachOrEvery<bool>) -> StructVar {
        Self::new(StructVarValue::Boolean(input))
    }
}

#[derive(Clone)]
pub struct StructPair(pub String,pub StructTemplate);

impl StructPair {
    pub fn new(key: &str, value: StructTemplate) -> StructPair {
        StructPair(key.to_string(),value)
    }
}

#[derive(Clone)]
pub enum StructTemplate {
    Var(StructVar),
    Const(StructConst),
    Array(Arc<EachOrEvery<StructTemplate>>),
    Object(Arc<EachOrEvery<StructPair>>),
    All(Vec<StructValueId>,Arc<StructTemplate>)
}


impl StructTemplate {
    pub fn new_var(input: StructVar) -> StructTemplate {
        if let Some(c) = input.to_const() {
            StructTemplate::Const(c)
        } else {
            StructTemplate::Var(input)
        }
    }

    pub fn new_all(vars: &[StructVar], expr: StructTemplate) -> StructTemplate {
        Self::All(vars.iter().map(|x| x.id).collect::<Vec<_>>(),Arc::new(expr))
    }

    pub fn new_number(input: f64) -> StructTemplate {
        Self::Const(StructConst::Number(input))
    }

    pub fn new_string(input: String) -> StructTemplate {
        Self::Const(StructConst::String(input))
    }

    pub fn new_boolean(input: bool) -> StructTemplate {
        Self::Const(StructConst::Boolean(input))
    }

    pub fn new_null() -> StructTemplate {
        Self::Const(StructConst::Null)
    }

    pub fn new_array(input: EachOrEvery<StructTemplate>) -> StructTemplate {
        Self::Array(Arc::new(input))
    }

    pub fn new_object(input: EachOrEvery<StructPair>) -> StructTemplate {
        Self::Object(Arc::new(input))
    }
}
