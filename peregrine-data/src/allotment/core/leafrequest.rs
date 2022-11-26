use std::{sync::{Arc, Mutex}, borrow::BorrowMut };
use peregrine_toolkit::lock;
use crate::{allotment::{transformers::drawinginfo::DrawingInfo, stylespec::stylegroup::AllotmentStyleGroup, boxes::leaf::FloatingLeaf}, LeafStyle};
use super::{allotmentname::{allotmentname_hashmap, AllotmentName, AllotmentNameHashMap}};

pub struct LeafTransformableMap {
    transformables: AllotmentNameHashMap<FloatingLeaf>
}

impl LeafTransformableMap {
    pub fn new() -> LeafTransformableMap {
        LeafTransformableMap {
            transformables: allotmentname_hashmap()
        }
    }

    pub(crate) fn set_transformable(&mut self, name: &AllotmentName, transformable: &FloatingLeaf) {
        self.transformables.insert(name.clone(),transformable.clone());
    }

    pub(crate) fn transformable(&self, name: &AllotmentName) -> &FloatingLeaf {
        self.transformables.get(name).unwrap()
    }
}

#[derive(Clone)]
pub struct LeafRequest {
    name: AllotmentName,
    drawing_info: Arc<Mutex<DrawingInfo>>,
    style: Arc<Mutex<Option<(Arc<AllotmentStyleGroup>,Arc<LeafStyle>)>>>
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
            drawing_info: Arc::new(Mutex::new(DrawingInfo::new())),
            style: Arc::new(Mutex::new(None))
        }
    }

    pub(crate) fn set_style(&self, style: &AllotmentStyleGroup) {
        let leaf = style.get_common_leaf(&self);
        *lock!(self.style) = Some((Arc::new(style.clone()),Arc::new(leaf)));
    }

    pub(crate) fn leaf_style(&self) -> Arc<LeafStyle> { lock!(self.style).as_ref().unwrap().1.clone() }
    pub(crate) fn style(&self) -> Arc<AllotmentStyleGroup> { lock!(self.style).as_ref().unwrap().0.clone() }
    pub fn name(&self) -> &AllotmentName {&self.name }

    pub(crate) fn drawing_info<F,T>(&self, mut cb: F) -> T where F: FnMut(&mut DrawingInfo) -> T {
        cb(lock!(self.drawing_info).borrow_mut())
    }
}
