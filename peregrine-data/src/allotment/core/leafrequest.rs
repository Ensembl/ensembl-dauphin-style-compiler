use std::{sync::{Arc, Mutex}, borrow::BorrowMut, collections::HashMap};

use peregrine_toolkit::lock;

use crate::{allotment::{transformers::drawinginfo::DrawingInfo, boxes::boxtraits::{Transformable}, stylespec::stylegroup::AllotmentStyleGroup, style::allotmentname::{AllotmentName, BuildPassThroughHasher, new_efficient_allotmentname_hashmap}}, LeafCommonStyle};

pub struct LeafTransformableMap {
    transformables: HashMap<AllotmentName,Arc<dyn Transformable>,BuildPassThroughHasher>
}

impl LeafTransformableMap {
    pub fn new() -> LeafTransformableMap {
        LeafTransformableMap {
            transformables: new_efficient_allotmentname_hashmap()
        }
    }

    pub(crate) fn set_transformable(&mut self, name: &AllotmentName, transformable: &Arc<dyn Transformable>) {
        self.transformables.insert(name.clone(),transformable.clone());
    }

    pub(crate) fn transformable(&self, name: &AllotmentName) -> &Arc<dyn Transformable> {
        self.transformables.get(name).unwrap()
    }
}

#[derive(Clone)]
pub struct LeafRequest {
    name: AllotmentName,
    drawing_info: Arc<Mutex<DrawingInfo>>,
    style: Arc<Mutex<Option<(Arc<AllotmentStyleGroup>,Arc<LeafCommonStyle>)>>>
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for LeafRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{:?}",self.name())
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

    pub fn leaf_style(&self) -> Arc<LeafCommonStyle> { lock!(self.style).as_ref().unwrap().1.clone() }
    pub fn style(&self) -> Arc<AllotmentStyleGroup> { lock!(self.style).as_ref().unwrap().0.clone() }
    pub fn name(&self) -> &AllotmentName {&self.name }
    pub fn drawing_info_clone(&self) -> DrawingInfo { lock!(self.drawing_info).clone() }
    pub fn update_drawing_info<F,T>(&self, mut cb: F) -> T where F: FnMut(&mut DrawingInfo) -> T {
        cb(lock!(self.drawing_info).borrow_mut())
    }
}
