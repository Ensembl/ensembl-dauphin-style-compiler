use std::sync::Arc;
use super::{eoestruct::{Struct, StructVisitor, StructConst, StructVarValue, VariableSystem, StructResult, StructError}, buildertree::BuiltVars, separatorvisitor::SeparatorVisitor};

#[cfg(debug_assertions)]
use super::eoestructformat::VariableSystemFormatter;

pub trait DataVisitor {
    fn visit_const(&mut self, _input: &StructConst) -> StructResult { Ok(()) }
    fn visit_separator(&mut self) {}
    fn visit_array_start(&mut self) -> StructResult { Ok(()) }
    fn visit_array_end(&mut self) -> StructResult { Ok(()) }
    fn visit_object_start(&mut self) -> StructResult { Ok(()) }
    fn visit_object_end(&mut self) -> StructResult { Ok(()) }
    fn visit_pair_start(&mut self, _key: &str) -> StructResult { Ok(()) }
    fn visit_pair_end(&mut self, _key: &str) -> StructResult { Ok(()) }
}

struct DataVisitorAdaptor<'a>(&'a mut dyn DataVisitor);

impl<'a,T: VariableSystem+Clone> StructVisitor<T> for DataVisitorAdaptor<'a> {
    fn visit_const(&mut self, input: &StructConst) -> StructResult { self.0.visit_const(input) }
    fn visit_array_start(&mut self) -> StructResult { self.0.visit_array_start() }
    fn visit_array_end(&mut self) -> StructResult { self.0.visit_array_end() }
    fn visit_object_start(&mut self) -> StructResult { self.0.visit_object_start() }
    fn visit_object_end(&mut self) -> StructResult { self.0.visit_object_end() }
    fn visit_pair_start(&mut self, key: &str) -> StructResult { self.0.visit_pair_start(key) }
    fn visit_pair_end(&mut self, key: &str) -> StructResult { self.0.visit_pair_end(key) }
}

impl<'a,T: VariableSystem+Clone> SeparatorVisitor<T> for DataVisitorAdaptor<'a> {
    fn visit_separator(&mut self) { self.0.visit_separator() }
}

#[derive(Clone)]
pub struct NullVars;

impl VariableSystem for NullVars {
    type Use = ();
    type Declare = ();

    #[cfg(debug_assertions)]
    fn build_formatter() -> Box<dyn VariableSystemFormatter<Self>> {
        Box::new(NullVarsFormatter)
    }
}

#[cfg(debug_assertions)]
pub struct NullVarsFormatter;

#[cfg(debug_assertions)]
impl VariableSystemFormatter<NullVars> for NullVarsFormatter {
    fn format_declare_start(&mut self, _var: &[()]) -> String {
        "<var!".to_string()
    }

    fn format_declare_end(&mut self, _var: &[()]) -> String {
        ">".to_string()
    }

    fn format_use(&mut self, _var: &()) -> Result<String,StructError> {
        Ok("use!".to_string())
    }
}

struct AllState {
    vars: Vec<Arc<StructVarValue>>,
    index: usize
}

impl AllState {
    fn new(vars: Vec<Arc<StructVarValue>>) -> AllState {
        AllState { vars, index: 0 }
    }

    fn get(&self, width: usize) -> StructConst {
        self.vars[width].get(self.index-1).unwrap() // guaranteed by build process
    }

    fn row(&mut self) -> bool {
        self.index += 1;
        self.vars[0].get(self.index-1).is_some()
    }
}

pub type StructBuilt = Struct<BuiltVars>;

impl StructBuilt {
    fn split(&self, output: &mut dyn StructVisitor<NullVars>, data: &mut Vec<AllState>) -> StructResult {
        match self {
            Struct::Var((depth,width)) => {
                output.visit_const(&data[*depth].get(*width))?;
            },
            Struct::Const(value) => {
                output.visit_const(value)?;
            },
            Struct::Array(values) => {
                output.visit_array_start()?;
                for value in values.iter() {
                    value.split(output,data)?;
                }
                output.visit_array_end()?;
            }
            Struct::Object(values) => {
                output.visit_object_start()?;
                for kv in values.iter() {
                    output.visit_pair_start(&kv.0)?;
                    kv.1.split(output,data)?;
                    output.visit_pair_end(&kv.0)?;
                }
                output.visit_object_end()?;
            },
            Struct::All(vars,expr) => {
                let all = AllState::new(vars.to_vec());
                data.push(all);
                output.visit_array_start()?;
                loop {
                    let top = data.last_mut().unwrap(); // data only manipulated here and just pushed
                    if !top.row() { break; }
                    drop(top);
                    expr.split(output,data)?;
                }
                output.visit_array_end()?;
                data.pop();
            }
        }
        Ok(())
    }

    pub(super) fn expand_struct(&self, output: &mut dyn StructVisitor<NullVars>) -> StructResult {
        self.split(output,&mut vec![])
    }

    pub fn expand(&self, output: &mut dyn DataVisitor) -> StructResult {
        self.expand_struct(&mut DataVisitorAdaptor(output))
    }
}
