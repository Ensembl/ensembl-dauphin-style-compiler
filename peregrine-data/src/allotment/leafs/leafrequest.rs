use std::{sync::{Arc, Mutex}, borrow::BorrowMut};
use peregrine_toolkit::lock;
use crate::allotment::{core::{allotmentname::AllotmentName}, style::{styletree::StyleTree, leafstyle::LeafStyle}};
use super::leafrequestbounds::LeafRequestBounds;

#[derive(Clone)]
pub struct LeafRequest {
    name: AllotmentName,
    shape_bounds: Arc<Mutex<LeafRequestBounds>>,
    style: Arc<Mutex<Option<(Arc<StyleTree>,Arc<LeafStyle>)>>>
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for LeafRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{:?}",self.name())
    }
}

impl PartialEq for LeafRequest {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for LeafRequest {}

impl std::hash::Hash for LeafRequest {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl LeafRequest {
    pub fn new(name: &AllotmentName) -> LeafRequest {
        LeafRequest {
            name: name.clone(),
            shape_bounds: Arc::new(Mutex::new(LeafRequestBounds::new())),
            style: Arc::new(Mutex::new(None))
        }
    }

    pub(crate) fn set_style(&self, style: &StyleTree) {
        let leaf = style.lookup_leaf(&self.name);
        *lock!(self.style) = Some((Arc::new(style.clone()),Arc::new(leaf.clone())));
    }

    pub(crate) fn leaf_style(&self) -> Arc<LeafStyle> { lock!(self.style).as_ref().unwrap().1.clone() }
    pub(crate) fn program_styles(&self) -> Arc<StyleTree> { lock!(self.style).as_ref().unwrap().0.clone() }
    pub fn name(&self) -> &AllotmentName { &self.name }

    pub(crate) fn shape_bounds<F,T>(&self, mut cb: F) -> T where F: FnMut(&mut LeafRequestBounds) -> T {
        cb(lock!(self.shape_bounds).borrow_mut())
    }
}
