use std::sync::Arc;
use crate::eachorevery::EachOrEveryGroupCompatible;
use super::{eoestruct::{VariableSystem, StructVarValue, Struct, StructValueId, StructConst, StructVisitor, StructResult, StructError, struct_error}, templatetree::{TemplateVars, StructVar}, buildstack::{BuildStack, IdentityBuildStackTransformer}, expand::StructBuilt};

#[cfg(debug_assertions)]
use super::eoestructformat::VariableSystemFormatter;

#[derive(Clone)]
pub struct BuiltVars;

impl VariableSystem for BuiltVars {
    type Declare = Arc<StructVarValue>;
    type Use = (usize,usize);

    #[cfg(debug_assertions)]
    fn build_formatter() -> Box<dyn VariableSystemFormatter<Self>> {
        Box::new(BuiltVarsFortmatter)
    }    
}

#[cfg(debug_assertions)]
struct BuiltVarsFortmatter;

#[cfg(debug_assertions)]
impl VariableSystemFormatter<BuiltVars> for BuiltVarsFortmatter {
    fn format_declare_start(&mut self, vars: &[Arc<StructVarValue>]) -> String {
        format!("A[{}].( ",vars.iter().map(|x| format!("{:?}",x)).collect::<Vec<_>>().join(""))
    }

    fn format_declare_end(&mut self, _var: &[Arc<StructVarValue>]) -> String {
        " )".to_string()
    }

    fn format_use(&mut self, var: &(usize,usize)) -> Result<String,StructError> {
        Ok(format!("D({},{})",var.0,var.1))
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

fn check_compatible(vars: &[Arc<StructVarValue>]) -> StructResult {
    if vars.len() == 0 {
        return Err(struct_error("no variables specified"));
    }
    let mut compat = EachOrEveryGroupCompatible::new(None);
    for var in vars {
        var.check_compatible(&mut compat);
    }
    if !compat.compatible() {
        return Err(struct_error("bindings of differing length"));
    }
    Ok(())
}

pub(super) struct TemplateBuildVisitor {
    all_depth: usize,
    build: BuildStack<StructBuilt,StructBuilt>,
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

    pub(super) fn get(self) -> StructBuilt { self.build.get() }
}

// XXX free and unknown
// XXX panics
impl StructVisitor<TemplateVars> for TemplateBuildVisitor {
    fn visit_const(&mut self, input: &StructConst) -> StructResult {
        self.build.add_atom(Struct::Const(input.clone()))?;
        Ok(())
    }

    fn visit_var(&mut self, input: &StructVar) -> StructResult {
        let index = self.bindings.iter().position(|id| id.id == input.id);
        let index = if let Some(x) = index { x } else { panic!("free"); };
        self.bindings[index].value = Some(input.value.clone());
        self.build.add_atom(Struct::Var(self.bindings[index].pos))
    }

    fn visit_array_start(&mut self) -> StructResult { self.build.push_array(); Ok(()) }
    fn visit_array_end(&mut self) -> StructResult { self.build.pop(|x| Ok(x)) }
    fn visit_object_start(&mut self) -> StructResult { self.build.push_object(); Ok(()) }
    fn visit_object_end(&mut self) -> StructResult { self.build.pop(|x| Ok(x)) }
    fn visit_pair_start(&mut self, key: &str) -> StructResult { self.build.push_key(key); Ok(()) }

    fn visit_all_start(&mut self, ids: &[StructValueId]) -> StructResult {
        for (i,id) in ids.iter().enumerate() {
            self.bindings.push(Binding::new(id,self.all_depth,i));
        }
        self.build.push_singleton();
        self.all_depth += 1;
        Ok(())
    }

    fn visit_all_end(&mut self, ids: &[StructValueId]) -> StructResult {
        let keep_len = self.bindings.len()-ids.len();
        let removed = self.bindings.split_off(keep_len);
        let removed = removed.iter().map(|binding| {
            Arc::new(binding.value.clone().expect("unset"))
        }).collect::<Vec<_>>();
        self.build.pop(|obj| {
            check_compatible(&removed)?;
            Ok(Struct::All(removed,Arc::new(obj)))
        })?;
        self.all_depth -= 1;
        Ok(())
    }
}
