use std::{sync::Arc, collections::hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use crate::eachorevery::EachOrEvery;
use super::eoestruct::{StructConst, StructVarValue};

#[derive(Clone)]
pub enum StructBuilt {
    Var(usize,usize),
    Const(StructConst),
    Array(Arc<EachOrEvery<StructBuilt>>,bool),
    Object(Arc<EachOrEvery<(String,StructBuilt)>>),
    All(Vec<Option<Arc<StructVarValue>>>,Arc<StructBuilt>),
    Condition(usize,usize,Arc<StructBuilt>)
}

impl Hash for StructBuilt {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            StructBuilt::Var(a,b) => { a.hash(state); b.hash(state); },
            StructBuilt::Const(c) => c.hash(state),
            StructBuilt::Array(a, _) => { a.hash(state); },
            StructBuilt::Object(b) => { b.hash(state); },
            StructBuilt::All(v,b) => { v.hash(state); b.hash(state); },
            StructBuilt::Condition(a,b,c) => { a.hash(state); b.hash(state); c.hash(state); },
        }
    }
}

impl PartialEq for StructBuilt {
    fn eq(&self, other: &Self) -> bool {
        let mut self_hash = DefaultHasher::new();
        self.hash(&mut self_hash);
        let mut other_hash = DefaultHasher::new();
        other.hash(&mut other_hash);
        self_hash.finish() == other_hash.finish()
    }
}

impl Eq for StructBuilt {}