use std::sync::Arc;

use crate::eachorevery::EachOrEveryGroupCompatible;

use super::{eoestruct::{VariableSystem, StructVarValue, Struct, StructPair, StructValueId, StructConst, StructVisitor}, eoestructformat::VariableSystemFormatter, templatetree::{TemplateVars, StructVar}};

#[derive(Clone)]
pub(super) struct BuiltVars;

impl VariableSystem for BuiltVars {
    type Declare = Arc<StructVarValue>;
    type Use = (usize,usize);

    fn build_formatter() -> Box<dyn VariableSystemFormatter<Self>> {
        Box::new(BuiltVarsFortmatter)
    }    
}

struct BuiltVarsFortmatter;

impl VariableSystemFormatter<BuiltVars> for BuiltVarsFortmatter {
    fn format_declare_start(&mut self, vars: &[Arc<StructVarValue>]) -> String {
        format!("A[{}].( ",vars.iter().map(|x| format!("{:?}",x)).collect::<Vec<_>>().join(""))
    }

    fn format_declare_end(&mut self, _var: &[Arc<StructVarValue>]) -> String {
        " )".to_string()
    }

    fn format_use(&mut self, var: &(usize,usize)) -> String {
        format!("D({},{})",var.0,var.1)
    }
}

enum TemplateBuildStackEntry {
    Node(Option<Struct<BuiltVars>>),
    Array(Vec<Struct<BuiltVars>>),
    Object(Vec<StructPair<BuiltVars>>)
}

struct Binding {
    id: StructValueId,
    pos: (usize,usize),
    value: Option<StructVarValue>
}

impl Binding {
    fn new(id: &StructValueId, depth: usize, width: usize) -> Binding {
        Binding { id: id.clone(), value: None, pos: (depth,width) }
    }
}

pub(super) struct TemplateBuildVisitor {
    all_depth: usize,
    stack: Vec<TemplateBuildStackEntry>,
    keys: Vec<String>,
    bindings: Vec<Binding>
}

impl TemplateBuildVisitor {
    pub(super) fn new() -> TemplateBuildVisitor {
        TemplateBuildVisitor {
            all_depth: 0,
            stack: vec![TemplateBuildStackEntry::Node(None)],
            keys: vec![],
            bindings: vec![]
        }
    }

    pub(super) fn get(mut self) -> Struct<BuiltVars> {
        if let TemplateBuildStackEntry::Node(Some(n) )= self.stack.pop().unwrap() {
            n
        } else {
            panic!("inocorrect stack size");
        }
    }

    fn add(&mut self, item: Struct<BuiltVars>) {
        match self.stack.last_mut().unwrap() {
            TemplateBuildStackEntry::Array(entries) => {
                entries.push(item);
            },
            TemplateBuildStackEntry::Object(entries) => {
                let key = self.keys.pop().unwrap();
                entries.push(StructPair(key,item));
            },
            TemplateBuildStackEntry::Node(value) => {
                *value = Some(item);
            }
        }
    }

    fn pop(&mut self) -> Struct<BuiltVars> {
        match self.stack.pop().unwrap() {
            TemplateBuildStackEntry::Array(entries) => {
                Struct::Array(Arc::new(entries))
            },
            TemplateBuildStackEntry::Object(entries) => {
                Struct::Object(Arc::new(entries))
            },
            TemplateBuildStackEntry::Node(node) => {
                node.expect("unset")
            }
        }
    }

    fn check_compatible(&self, vars: &[Arc<StructVarValue>]) {
        let mut compat = EachOrEveryGroupCompatible::new(None);
        for var in vars {
            var.check_compatible(&mut compat);
        }

    }
}

// XXX panics
impl StructVisitor<TemplateVars> for TemplateBuildVisitor {
    fn visit_const(&mut self, input: &StructConst) {
        self.add(Struct::Const(input.clone()));
    }

    fn visit_var(&mut self, input: &StructVar) {
        let index = self.bindings.iter().position(|id| id.id == input.id);
        let index = if let Some(x) = index { x } else { panic!("free"); };
        self.bindings[index].value = Some(input.value.clone());
        self.add(Struct::Var(self.bindings[index].pos));
    }

    fn visit_array_start(&mut self) {
        self.stack.push(TemplateBuildStackEntry::Array(vec![]));
    }

    fn visit_array_end(&mut self) {
        let obj = self.pop();
        self.add(obj);
    }

    fn visit_object_start(&mut self) {
        self.stack.push(TemplateBuildStackEntry::Object(vec![]));
    }

    fn visit_object_end(&mut self) {
        let obj = self.pop();
        self.add(obj);
    }

    fn visit_pair_start(&mut self, key: &str) {
        self.keys.push(key.to_string());
    }

    fn visit_all_start(&mut self, ids: &[StructValueId]) {
        for (i,id) in ids.iter().enumerate() {
            self.bindings.push(Binding::new(id,self.all_depth,i));
        }
        self.stack.push(TemplateBuildStackEntry::Node(None));
        self.all_depth += 1;
    }

    fn visit_all_end(&mut self, ids: &[StructValueId]) {
        let keep_len = self.bindings.len()-ids.len();
        let removed = self.bindings.split_off(keep_len);
        let removed = removed.iter().map(|binding| {
            Arc::new(binding.value.clone().expect("unset"))
        }).collect::<Vec<_>>();
        let obj = self.pop();
        self.check_compatible(&removed);
        self.add(Struct::All(removed,Arc::new(obj)));
        self.all_depth -= 1;
    }
}
