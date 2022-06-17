use std::sync::Arc;
use super::{eoestruct::{Struct, StructVisitor, StructConst, StructVarValue, VariableSystem}, buildertree::BuiltVars, eoestructformat::VariableSystemFormatter};

#[derive(Clone)]
pub struct NullVars;

impl VariableSystem for NullVars {
    type Use = ();
    type Declare = ();

    fn build_formatter() -> Box<dyn VariableSystemFormatter<Self>> {
        Box::new(NullVarsFormatter)
    }
}

pub struct NullVarsFormatter;

impl VariableSystemFormatter<NullVars> for NullVarsFormatter {
    fn format_declare_start(&mut self, _var: &[()]) -> String {
        "<var!".to_string()
    }

    fn format_declare_end(&mut self, _var: &[()]) -> String {
        ">".to_string()
    }

    fn format_use(&mut self, _var: &()) -> String {
        "use!".to_string()
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
        self.vars[width].get(self.index-1).unwrap()
    }

    // XXX zero all
    fn row(&mut self) -> bool {
        self.index += 1;
        self.vars[0].get(self.index-1).is_some()
    }
}

struct ExpandData {
    alls: Vec<AllState>
}

impl ExpandData {
    fn new() -> ExpandData {
        ExpandData {
            alls: vec![]
        }
    }
}

impl Struct<BuiltVars> {
    fn split(&self, output: &mut dyn StructVisitor<NullVars>, data: &mut ExpandData) {
        match self {
            Struct::Var((depth,width)) => {
                output.visit_const(&data.alls[*depth].get(*width));
            },
            Struct::Const(value) => {
                output.visit_const(value);
            },
            Struct::Array(values) => {
                output.visit_array_start();
                for value in values.iter() {
                    value.split(output,data);
                }
                output.visit_array_end();
            }
            Struct::Object(values) => {
                output.visit_object_start();
                for kv in values.iter() {
                    output.visit_pair_start(&kv.0);
                    kv.1.split(output,data);
                    output.visit_pair_end(&kv.0);
                }
                output.visit_object_end();
            },
            Struct::All(vars,expr) => {
                let all = AllState::new(vars.to_vec());
                data.alls.push(all);
                output.visit_array_start();
                loop {
                    let top = data.alls.last_mut().unwrap();
                    if !top.row() { break; }
                    drop(top);
                    expr.split(output,data);
                }
                output.visit_array_end();
                data.alls.pop();
            }
        }
    }

    pub fn expand(&self, output: &mut dyn StructVisitor<NullVars>) {
        self.split(output,&mut ExpandData::new())
    }
}
