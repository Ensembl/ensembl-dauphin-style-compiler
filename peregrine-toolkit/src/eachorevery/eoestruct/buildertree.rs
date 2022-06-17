use std::sync::Arc;

use crate::eachorevery::EachOrEveryGroupCompatible;

use super::{eoestruct::{VariableSystem, StructVarValue, Struct, StructValueId, StructConst, StructVisitor}, eoestructformat::VariableSystemFormatter, templatetree::{TemplateVars, StructVar}, buildstack::{BuildStack, IdentityBuildStackTransformer}};

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

fn check_compatible(vars: &[Arc<StructVarValue>]) {
    let mut compat = EachOrEveryGroupCompatible::new(None);
    for var in vars {
        var.check_compatible(&mut compat);
    }

}

pub(super) struct TemplateBuildVisitor {
    all_depth: usize,
    build: BuildStack<Struct<BuiltVars>,Struct<BuiltVars>>,
    bindings: Vec<Binding>
}

impl TemplateBuildVisitor {
    pub(super) fn new() -> TemplateBuildVisitor {
        TemplateBuildVisitor {
            all_depth: 0,
            build: BuildStack::new(IdentityBuildStackTransformer),
            bindings: vec![]
        }
    }

    pub(super) fn get(mut self) -> Struct<BuiltVars> { self.build.get() }
}

// XXX panics
impl StructVisitor<TemplateVars> for TemplateBuildVisitor {
    fn visit_const(&mut self, input: &StructConst) {
        self.build.add_atom(Struct::Const(input.clone()));
    }

    fn visit_var(&mut self, input: &StructVar) {
        let index = self.bindings.iter().position(|id| id.id == input.id);
        let index = if let Some(x) = index { x } else { panic!("free"); };
        self.bindings[index].value = Some(input.value.clone());
        self.build.add_atom(Struct::Var(self.bindings[index].pos));
    }

    fn visit_array_start(&mut self) { self.build.push_array(); }
    fn visit_array_end(&mut self) { self.build.pop(|x| x); }
    fn visit_object_start(&mut self) { self.build.push_object(); }
    fn visit_object_end(&mut self) { self.build.pop(|x| x); }
    fn visit_pair_start(&mut self, key: &str) { self.build.push_key(key); }

    fn visit_all_start(&mut self, ids: &[StructValueId]) {
        for (i,id) in ids.iter().enumerate() {
            self.bindings.push(Binding::new(id,self.all_depth,i));
        }
        self.build.push_singleton();
        self.all_depth += 1;
    }

    fn visit_all_end(&mut self, ids: &[StructValueId]) {
        let keep_len = self.bindings.len()-ids.len();
        let removed = self.bindings.split_off(keep_len);
        let removed = removed.iter().map(|binding| {
            Arc::new(binding.value.clone().expect("unset"))
        }).collect::<Vec<_>>();
        self.build.pop(|obj| {
            check_compatible(&removed);
            Struct::All(removed,Arc::new(obj))
        });
        self.all_depth -= 1;
    }
}
