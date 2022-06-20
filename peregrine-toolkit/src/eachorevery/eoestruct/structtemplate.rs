use std::sync::Arc;
use crate::eachorevery::EachOrEvery;
use super::{eoestruct::{StructConst, StructValueId, StructVarValue, StructVarGroup}};

#[derive(Clone)]
pub struct StructVar {
    pub(super) value: StructVarValue,
    pub(super) id: StructValueId
}

impl StructVar {
    fn to_const(&self) -> Option<StructConst> { self.value.to_const() }

    fn new(group:&mut StructVarGroup, value: StructVarValue) -> StructVar {
        let id = StructValueId::new();
        group.0.push(id.clone());
        StructVar { value, id }
    }

    pub fn new_number(group:&mut StructVarGroup, input: EachOrEvery<f64>) -> StructVar {
        Self::new(group,StructVarValue::Number(input))
    }

    pub fn new_string(group:&mut StructVarGroup, input: EachOrEvery<String>) -> StructVar {
        Self::new(group,StructVarValue::String(input))
    }

    pub fn new_boolean(group:&mut StructVarGroup, input: EachOrEvery<bool>) -> StructVar {
        Self::new(group,StructVarValue::Boolean(input))
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
    All(Vec<StructValueId>,Arc<StructTemplate>),
    Condition(StructVar,Arc<StructTemplate>)
}

impl StructTemplate {
    pub fn new_var(input: StructVar) -> StructTemplate {
        if let Some(c) = input.to_const() {
            StructTemplate::Const(c)
        } else {
            StructTemplate::Var(input)
        }
    }

    pub fn new_all(vars: StructVarGroup, expr: StructTemplate) -> StructTemplate {
        Self::All(vars.0.clone(),Arc::new(expr))
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

    pub fn new_condition(input: StructVar, expr: StructTemplate) -> StructTemplate {
        Self::Condition(input,Arc::new(expr))
    }
}
