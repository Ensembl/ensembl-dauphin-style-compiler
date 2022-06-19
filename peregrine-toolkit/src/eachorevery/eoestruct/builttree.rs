use std::sync::Arc;

use super::eoestruct::{StructConst, StructVarValue};

#[derive(Clone)]
pub enum StructBuilt {
    Var(usize,usize),
    Const(StructConst),
    Array(Arc<Vec<StructBuilt>>),
    Object(Arc<Vec<(String,StructBuilt)>>),
    All(Vec<Arc<StructVarValue>>,Arc<StructBuilt>)
}
