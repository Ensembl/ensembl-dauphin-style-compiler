use std::sync::Arc;
use crate::eachorevery::EachOrEveryGroupCompatible;
use super::{eoestruct::{VariableSystem, StructVarValue, Struct, StructValueId, StructResult, StructError, struct_error, StructPair}, expand::StructBuilt, StructTemplate};

#[cfg(debug_assertions)]
use super::eoedebug::VariableSystemFormatter;

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

impl StructTemplate {
    fn make(&self, bindings: &mut Vec<Binding>, all_depth: usize) -> Result<StructBuilt,StructError> {
        Ok(match self {
            Struct::Var(var) => {
                let index = bindings.iter().position(|id| id.id == var.id);
                let index = index.ok_or_else(|| struct_error("free variable in template"))?;
                bindings[index].value = Some(var.value.clone());
                Struct::Var(bindings[index].pos)
            },
            Struct::Const(c) => { Struct::Const(c.clone()) }
            Struct::Array(v) => {
                Struct::Array(Arc::new(
                    v.iter().map(|x| 
                        x.make(bindings,all_depth)
                    ).collect::<Result<_,_>>()?
                ))
            },
            Struct::Object(v) => {
                Struct::Object(Arc::new(v.iter().map(|x| 
                        Ok(StructPair(x.0.clone(),x.1.make(bindings,all_depth)?))
                    ).collect::<Result<Vec<_>,String>>()?
                ))
            },
            Struct::All(ids, expr) => {
                for (i,id) in ids.iter().enumerate() {
                    bindings.push(Binding::new(id,all_depth,i));
                }
                let obj = expr.make(bindings,all_depth+1)?;
                let keep_len = bindings.len()-ids.len();
                let removed = bindings.split_off(keep_len);
                let removed = removed.iter().filter_map(|binding| {
                    binding.value.clone().map(|x| Arc::new(x))
                }).collect::<Vec<_>>();
                if removed.is_empty() {
                    Struct::Array(Arc::new(vec![obj]))
                } else {
                    check_compatible(&removed)?;
                    Struct::All(removed,Arc::new(obj))
                }
            }
        })
    }

    pub fn build(&self) -> Result<StructBuilt,StructError> {
        self.make(&mut vec![],0)
    }
}
