use std::sync::Arc;
use crate::eachorevery::EachOrEvery;
use super::{eoestruct::{StructConst, StructVarValue, StructResult, struct_error, StructError, LateValues}, eoestructdata::DataVisitor, structbuilt::StructBuilt};

fn separate<'a,F,Y>(input: &EachOrEvery<Y>, mut cb: F, visitor: &mut dyn DataVisitor) -> StructResult
        where F: FnMut(&Y,&mut dyn DataVisitor) -> StructResult {
    let mut first = true;
    if let Some(len) = input.len() {
        for item in input.iter(len).unwrap() {
            if !first { visitor.visit_separator()?; }
            cb(item,visitor)?;
            first = false;
        }
    } else {
        return Err(struct_error("expanding infinitely"));
    }
    Ok(())        
}

struct AllState {
    vars: Vec<Option<Arc<StructVarValue>>>,
    index: usize,
    first: usize
}

struct GlobalState<'a> {
    lates: Option<&'a LateValues>,
    alls: Vec<AllState>
}

impl AllState {
    fn new(vars: Vec<Option<Arc<StructVarValue>>>, lates: Option<&LateValues>) -> Result<AllState,StructError> {
        let first = vars.iter().position(|x| 
            x.as_ref().map(|x| x.is_finite(lates).ok().unwrap_or(false)).unwrap_or(false)
        ).ok_or_else(|| struct_error("no infinite recursion allowed"))?;
        Ok(AllState { vars, index: 0, first })
    }

    fn get(&self, lates: Option<&LateValues>, width: usize) -> Result<StructConst,StructError> {
        self.vars[width].as_ref().unwrap().get(lates,self.index-1)
    }

    fn row(&mut self, lates: Option<&LateValues>) -> Result<bool,StructError> {
        self.index += 1;
        self.vars[self.first].as_ref().unwrap().exists(lates,self.index-1)
    }
}

impl StructBuilt {
    fn split(&self, output: &mut dyn DataVisitor, data: &mut GlobalState) -> StructResult {
        match self {
            StructBuilt::Var(depth,width) => {
                output.visit_const(&data.alls[*depth].get(data.lates,*width)?)?;
            },
            StructBuilt::Const(value) => {
                output.visit_const(value)?;
            },
            StructBuilt::Array(values) => {
                output.visit_array_start()?;
                separate(&values,|value,visitor| {
                    value.split(visitor,data)
                },output)?;
                output.visit_array_end()?;
            }
            StructBuilt::Object(values) => {
                output.visit_object_start()?;
                separate(&values, |kv,visitor| {
                    visitor.visit_pair_start(&kv.0)?;
                    kv.1.split(visitor,data)?;
                    visitor.visit_pair_end(&kv.0)
                },output)?;
                output.visit_object_end()?;
            },
            StructBuilt::All(vars,expr) => {
                let all = AllState::new(vars.to_vec(),data.lates)?;
                data.alls.push(all);
                output.visit_array_start()?;
                let mut first = true;
                loop {
                    let top = data.alls.last_mut().unwrap(); // data only manipulated here and just pushed
                    if !top.row(data.lates)? { break; }
                    if !first { output.visit_separator()?; }
                    expr.split(output,data)?;
                    first = false;
                }
                output.visit_array_end()?;
                data.alls.pop();
            }
            StructBuilt::Condition(depth,width,expr) => {
                if data.alls[*depth].get(data.lates,*width)?.truthy() {
                    expr.split(output,data)?;
                }
            }
        }
        Ok(())
    }

    pub fn expand(&self, lates: Option<&LateValues>, output: &mut dyn DataVisitor) -> StructResult {
        self.split(output,&mut GlobalState { alls: vec![], lates })
    }
}
