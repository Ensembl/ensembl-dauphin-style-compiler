use std::{collections::HashMap, sync::Arc};

use crate::eachorevery::EachOrEvery;

use super::{eoestruct::{StructConst, Struct, StructPair, StructValueId, VariableSystem, StructVarValue}, eoestructformat::VariableSystemFormatter, buildertree::{TemplateBuildVisitor, BuiltVars}};

#[derive(Clone)]
pub struct StructVar {
    pub(super) value: StructVarValue,
    pub(super) id: StructValueId
}

pub(super) trait Visitable<T> {
    fn visit(&mut self, value: &T);
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

impl std::fmt::Debug for StructVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}={:?}",self.id.0,self.value)
    }
}

#[derive(Clone)]
pub(super) struct TemplateVars;

impl VariableSystem for TemplateVars {
    type Declare = StructValueId;
    type Use = StructVar;

    fn build_formatter() -> Box<dyn VariableSystemFormatter<Self>> {
        Box::new(TemplateVarsFormatter::new())
    }    
}

impl StructPair<TemplateVars> {
    pub fn new(key: &str, value: Struct<TemplateVars>) -> StructPair<TemplateVars> {
        StructPair(key.to_string(),value)
    }
}

struct TemplateVarsFormatter {
    name: HashMap<StructValueId,usize>
}

impl TemplateVarsFormatter {
    pub(super) fn new() -> TemplateVarsFormatter {
        TemplateVarsFormatter {
            name: HashMap::new()
        }
    }

    fn get(&mut self, value: &StructValueId) -> String {
        let len = self.name.len();
        let index = *self.name.entry(*value).or_insert(len);
        let vars = ('a'..'z').collect::<String>();
        let series = index / (vars.len());
        let series = if series > 0 { format!("{}",series) } else { "".to_string() };
        let offset = index % (vars.len());
        format!("{}{}",series,vars.chars().nth(offset).unwrap())
    }
}

impl VariableSystemFormatter<TemplateVars> for TemplateVarsFormatter {
    fn format_declare_start(&mut self, vars: &[StructValueId]) -> String {
        format!("A{}.( ",vars.iter().map(|x| self.get(x)).collect::<Vec<_>>().join(""))
    }

    fn format_declare_end(&mut self, _vars: &[StructValueId]) -> String {
        " )".to_string()
    }

    fn format_use(&mut self, var: &StructVar) -> String {
        format!("{}={:?}",self.get(&var.id),var.value)
    }
}

impl Struct<TemplateVars> {
    pub fn new_var(input: StructVar) -> Struct<TemplateVars> {
        if let Some(c) = input.to_const() {
            Struct::Const(c)
        } else {
            Struct::Var(input)
        }
    }

    pub fn new_all(vars: &[StructVar], expr: Struct<TemplateVars>) -> Struct<TemplateVars> {
        Self::All(vars.iter().map(|x| x.id).collect::<Vec<_>>(),Arc::new(expr))
    }

    pub fn splitter(&self) -> Struct<BuiltVars> {
        let mut builder = TemplateBuildVisitor::new();
        self.visit(&mut builder);
        builder.get()
    }

    pub fn new_number(input: f64) -> Struct<TemplateVars> {
        Self::Const(StructConst::Number(input))
    }

    pub fn new_string(input: String) -> Struct<TemplateVars> {
        Self::Const(StructConst::String(input))
    }

    pub fn new_boolean(input: bool) -> Struct<TemplateVars> {
        Self::Const(StructConst::Boolean(input))
    }

    pub fn new_array(input: Vec<Struct<TemplateVars>>) -> Struct<TemplateVars> {
        Self::Array(Arc::new(input))
    }

    pub fn new_object(input: Vec<StructPair<TemplateVars>>) -> Struct<TemplateVars> {
        Self::Object(Arc::new(input))
    }
}
