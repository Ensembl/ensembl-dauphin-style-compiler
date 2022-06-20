use std::sync::Arc;
use crate::eachorevery::EachOrEvery;
use super::{eoestruct::{StructConst, StructVarValue, StructResult, struct_error}, eoestructdata::DataVisitor, structbuilt::StructBuilt};

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

impl AllState {
    fn new(vars: Vec<Option<Arc<StructVarValue>>>) -> AllState {
        let first = vars.iter().position(|x| x.is_some()).unwrap();
        AllState { vars, index: 0, first }
    }

    fn get(&self, width: usize) -> StructConst {
        self.vars[width].as_ref().unwrap().get(self.index-1).unwrap() // guaranteed by build process
    }

    fn row(&mut self) -> bool {
        self.index += 1;
        self.vars[self.first].as_ref().unwrap().get(self.index-1).is_some()
    }
}

impl StructBuilt {
    fn split(&self, output: &mut dyn DataVisitor, data: &mut Vec<AllState>) -> StructResult {
        match self {
            StructBuilt::Var(depth,width) => {
                output.visit_const(&data[*depth].get(*width))?;
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
            StructBuilt::Condition(depth,width,expr) => {
                if data[*depth].get(*width).truthy() {
                    expr.split(output,data)?;
                }
            }
        }
        Ok(())
    }

    pub fn expand(&self, output: &mut dyn DataVisitor) -> StructResult {
        self.split(output,&mut vec![])
    }
}
