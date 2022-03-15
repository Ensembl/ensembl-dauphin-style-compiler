use std::{sync::{Arc, Mutex}, borrow::BorrowMut};

use peregrine_toolkit::lock;

use crate::{allotment::{transformers::drawinginfo::DrawingInfo, boxes::boxtraits::Transformable, stylespec::stylegroup::AllotmentStyleGroup}, LeafCommonStyle};
use super::allotmentname::AllotmentName;

#[derive(Clone)]
pub struct PendingLeaf {
    name: AllotmentName,
    drawing_info: Arc<Mutex<DrawingInfo>>,
    style: Arc<Mutex<Option<(Arc<AllotmentStyleGroup>,Arc<LeafCommonStyle>)>>>,
    transformable: Arc<Mutex<Option<Arc<dyn Transformable>>>>
}

impl std::fmt::Debug for PendingLeaf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{:?}",self.name())
    }
}

impl PendingLeaf {
    pub fn new(name: &AllotmentName) -> PendingLeaf {
        PendingLeaf {
            name: name.clone(),
            drawing_info: Arc::new(Mutex::new(DrawingInfo::new())),
            style: Arc::new(Mutex::new(None)),
            transformable: Arc::new(Mutex::new(None))
        }
    }

    pub(crate) fn set_style(&self, style: &AllotmentStyleGroup) {
        let leaf = style.get_common_leaf(&self);
        *lock!(self.style) = Some((Arc::new(style.clone()),Arc::new(leaf)));
    }

    pub(crate) fn set_transformable(&self, xformable: Arc<dyn Transformable>) {
        *lock!(self.transformable) = Some(xformable);
    }

    /* only call after set_transformable has been called! (via make_transformable) */
    pub fn transformable(&self) -> Arc<dyn Transformable> { lock!(self.transformable).as_ref().unwrap().clone() }

    pub fn leaf_style(&self) -> Arc<LeafCommonStyle> { lock!(self.style).as_ref().unwrap().1.clone() }
    pub fn style(&self) -> Arc<AllotmentStyleGroup> { lock!(self.style).as_ref().unwrap().0.clone() }
    pub fn name(&self) -> &AllotmentName {&self.name }
    pub fn drawing_info_clone(&self) -> DrawingInfo { lock!(self.drawing_info).clone() }
    pub fn update_drawing_info<F,T>(&self, mut cb: F) -> T where F: FnMut(&mut DrawingInfo) -> T {
        cb(lock!(self.drawing_info).borrow_mut())
    }
}
