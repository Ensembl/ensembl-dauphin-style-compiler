use std::sync::Arc;
use crate::eachorevery::{EachOrEveryGroupCompatible, EachOrEvery};
use super::{eoestruct::{StructVarValue, StructValueId, StructResult, StructError, struct_error}, StructTemplate, structbuilt::StructBuilt};

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
            StructTemplate::Var(var) => {
                let index = bindings.iter().position(|id| id.id == var.id);
                let index = index.ok_or_else(|| struct_error("free variable in template"))?;
                bindings[index].value = Some(var.value.clone());
                let pos = bindings[index].pos;
                StructBuilt::Var(pos.0,pos.1)
            },
            StructTemplate::Const(c) => { StructBuilt::Const(c.clone()) }
            StructTemplate::Array(v) => {
                if v.len().is_none() {
                    return Err(struct_error("no infinite arrays in json"));
                }
                StructBuilt::Array(Arc::new(
                    v.map_results(|x| x.make(bindings,all_depth))?
                ))
            },
            StructTemplate::Object(v) => {
                if v.len().is_none() {
                    return Err(struct_error("no infinite objects in json"));
                }
                StructBuilt::Object(Arc::new(
                    v.map_results::<_,_,StructError>(|x| Ok((x.0.clone(),x.1.make(bindings,all_depth)?)))?
                ))
            },
            StructTemplate::All(ids, expr) => {
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
                    StructBuilt::Array(Arc::new(EachOrEvery::each(vec![obj])))
                } else {
                    check_compatible(&removed)?;
                    StructBuilt::All(removed,Arc::new(obj))
                }
            }
            StructTemplate::Condition(var,expr) => {
                let index = bindings.iter().position(|id| id.id == var.id);
                let index = index.ok_or_else(|| struct_error("free variable in template"))?;
                bindings[index].value = Some(var.value.clone());
                let pos = bindings[index].pos;
                let expr = expr.make(bindings,all_depth)?;
                StructBuilt::Condition(pos.0,pos.1,Arc::new(expr))
            }
        })
    }

    pub fn build(&self) -> Result<StructBuilt,StructError> {
        self.make(&mut vec![],0)
    }
}
