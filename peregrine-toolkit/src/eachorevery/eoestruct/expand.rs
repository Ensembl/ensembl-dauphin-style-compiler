use std::sync::Arc;
use super::{eoestruct::{StructConst, StructVarValue, StructResult}, eoestructdata::DataVisitor, builttree::StructBuilt};

fn separate<'a,F,X,Y>(it: X, mut cb: F, visitor: &mut dyn DataVisitor) -> StructResult
        where X: Iterator<Item=Y>,
              F: FnMut(Y,&mut dyn DataVisitor) -> StructResult {
    let mut first = true;
    for item in it {
        if !first { visitor.visit_separator()?; }
        cb(item,visitor)?;
        first = false;
    }
    Ok(())
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
                separate(values.iter(),|value,visitor| {
                    value.split(visitor,data)
                },output)?;
                output.visit_array_end()?;
            }
            StructBuilt::Object(values) => {
                output.visit_object_start()?;
                separate(values.iter(), |kv,visitor| {
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
        }
        Ok(())
    }

    pub fn expand(&self, output: &mut dyn DataVisitor) -> StructResult {
        self.split(output,&mut vec![])
    }
}
