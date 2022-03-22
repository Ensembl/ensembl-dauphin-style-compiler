use std::{sync::{Arc, Mutex}, borrow::BorrowMut};

use peregrine_toolkit::lock;

use crate::{allotment::{transformers::drawinginfo::DrawingInfo, boxes::boxtraits::{Transformable, DustbinTransformable}, stylespec::stylegroup::AllotmentStyleGroup}, LeafCommonStyle};
use super::allotmentname::AllotmentName;

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub struct PendingLeafSource {
    next_index: Arc<Mutex<usize>>,
    #[cfg(debug_assertions)]
    closed: Arc<Mutex<bool>>
}

impl PendingLeafSource {
    pub fn new() -> PendingLeafSource {
        PendingLeafSource {
            next_index: Arc::new(Mutex::new(0)),
            #[cfg(debug_assertions)]
            closed: Arc::new(Mutex::new(false))
        }
    }

    pub fn id(&self) -> usize {
        #[cfg(debug_assertions)]
        if *lock!(self.closed) {
            panic!("allocate from closed!");
        }
        let mut index = lock!(self.next_index);
        *index += 1;
        *index - 1
    }
}

pub struct PendingLeafMap {
    transformables: Vec<Arc<dyn Transformable>>
}

impl PendingLeafMap {
    pub fn new(source: &PendingLeafSource) -> PendingLeafMap {
        #[cfg(debug_assertions)]
        { *lock!(source.closed) = true; }
        let index_len = *lock!(source.next_index);
        let dustbin = Arc::new(DustbinTransformable::new());
        PendingLeafMap {
            transformables: vec![dustbin;index_len]
        }
    }

    fn set_transformable(&mut self, index: usize, transformable: &Arc<dyn Transformable>) {
        self.transformables[index] = transformable.clone();
    }

    fn transformable(&self, index: usize) -> &Arc<dyn Transformable> {
        &self.transformables[index]
    }
}

#[derive(Clone)]
pub struct PendingLeaf {
    name: AllotmentName,
    index: Arc<Mutex<usize>>,
    drawing_info: Arc<Mutex<DrawingInfo>>,
    style: Arc<Mutex<Option<(Arc<AllotmentStyleGroup>,Arc<LeafCommonStyle>)>>>
}

impl std::fmt::Debug for PendingLeaf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{:?}",self.name())
    }
}

impl PendingLeaf {
    pub fn new(source: &mut PendingLeafSource, name: &AllotmentName) -> PendingLeaf {
        PendingLeaf {
            name: name.clone(),
            index: Arc::new(Mutex::new(source.id())),
            drawing_info: Arc::new(Mutex::new(DrawingInfo::new())),
            style: Arc::new(Mutex::new(None))
        }
    }

    pub fn new_source(&mut self, source: &mut PendingLeafSource) {
        *lock!(self.index) = source.id();
    }

    pub(crate) fn set_style(&self, style: &AllotmentStyleGroup) {
        let leaf = style.get_common_leaf(&self);
        *lock!(self.style) = Some((Arc::new(style.clone()),Arc::new(leaf)));
    }

    pub(crate) fn set_transformable(&self, plm: &mut PendingLeafMap, xformable: Arc<dyn Transformable>) {
        plm.set_transformable(*lock!(self.index),&xformable);
    }

    /* only call after set_transformable has been called! (via make_transformable) */
    pub fn transformable(&self, plm: &PendingLeafMap) -> Arc<dyn Transformable> { 
        plm.transformable(*lock!(self.index)).clone()
    }

    pub fn leaf_style(&self) -> Arc<LeafCommonStyle> { lock!(self.style).as_ref().unwrap().1.clone() }
    pub fn style(&self) -> Arc<AllotmentStyleGroup> { lock!(self.style).as_ref().unwrap().0.clone() }
    pub fn name(&self) -> &AllotmentName {&self.name }
    pub fn drawing_info_clone(&self) -> DrawingInfo { lock!(self.drawing_info).clone() }
    pub fn update_drawing_info<F,T>(&self, mut cb: F) -> T where F: FnMut(&mut DrawingInfo) -> T {
        cb(lock!(self.drawing_info).borrow_mut())
    }
}
