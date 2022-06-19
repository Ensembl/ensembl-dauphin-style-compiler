use std::sync::Arc;

use crate::eachorevery::EachOrEvery;

use super::eoestruct::{StructConst, StructVarValue};

#[derive(Clone)]
pub enum StructBuilt {
    Var(usize,usize),
    Const(StructConst),
    Array(Arc<EachOrEvery<StructBuilt>>),
    Object(Arc<EachOrEvery<(String,StructBuilt)>>),
    All(Vec<Arc<StructVarValue>>,Arc<StructBuilt>)
}
