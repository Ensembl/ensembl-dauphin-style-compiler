use std::sync::Arc;
use super::{eoestruct::{Struct, StructConst, StructVarValue, StructResult}, buildertree::BuiltVars};

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

pub trait DataVisitor {
    fn visit_const(&mut self, _input: &StructConst) -> StructResult { Ok(()) }
    fn visit_separator(&mut self) -> StructResult { Ok(()) }
    fn visit_array_start(&mut self) -> StructResult { Ok(()) }
    fn visit_array_end(&mut self) -> StructResult { Ok(()) }
    fn visit_object_start(&mut self) -> StructResult { Ok(()) }
    fn visit_object_end(&mut self) -> StructResult { Ok(()) }
    fn visit_pair_start(&mut self, _key: &str) -> StructResult { Ok(()) }
    fn visit_pair_end(&mut self, _key: &str) -> StructResult { Ok(()) }
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
    fn split(&self, output: &mut dyn DataVisitor, data: &mut Vec<AllState>) -> StructResult {
        match self {
            Struct::Var((depth,width)) => {
                output.visit_const(&data[*depth].get(*width))?;
            },
            Struct::Const(value) => {
                output.visit_const(value)?;
            },
            Struct::Array(values) => {
                output.visit_array_start()?;
                separate(values.iter(),|value,visitor| {
                    value.split(visitor,data)
                },output)?;
                output.visit_array_end()?;
            }
            Struct::Object(values) => {
                output.visit_object_start()?;
                separate(values.iter(), |kv,visitor| {
                    visitor.visit_pair_start(&kv.0)?;
                    kv.1.split(visitor,data)?;
                    visitor.visit_pair_end(&kv.0)
                },output)?;
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

    pub(super) fn expand_struct(&self, output: &mut dyn DataVisitor) -> StructResult {
        self.split(output,&mut vec![])
    }

    pub fn expand(&self, output: &mut dyn DataVisitor) -> StructResult {
        self.expand_struct(output)
    }
}
