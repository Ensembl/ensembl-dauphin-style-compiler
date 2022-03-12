use std::{sync::{Arc, Mutex}, borrow::BorrowMut};

use peregrine_toolkit::lock;

use crate::{allotment::{transformers::drawinginfo::DrawingInfo, boxes::boxtraits::Transformable}};
use super::allotmentname::AllotmentName;

#[derive(Clone)]
pub struct PendingLeaf {
    name: AllotmentName,
    drawing_info: Arc<Mutex<DrawingInfo>>,
    transformable: Arc<Mutex<Option<Arc<dyn Transformable>>>>
}

impl PendingLeaf {
    pub fn new(name: &AllotmentName) -> PendingLeaf {
        PendingLeaf {
            name: name.clone(),
            drawing_info: Arc::new(Mutex::new(DrawingInfo::new())),
            transformable: Arc::new(Mutex::new(None))
        }
    }

    pub(crate) fn set_transformable(&self, xformable: Arc<dyn Transformable>) {
        *lock!(self.transformable) = Some(xformable);
    }

    /* only call after set_transformable has been called! (via make_transformable) */
    pub fn transformable(&self) -> Arc<dyn Transformable> { lock!(self.transformable).as_ref().unwrap().clone() }
    
    pub fn name(&self) -> &AllotmentName {&self.name }
    pub fn drawing_info_clone(&self) -> DrawingInfo { lock!(self.drawing_info).clone() }
    pub fn update_drawing_info<F,T>(&self, mut cb: F) -> T where F: FnMut(&mut DrawingInfo) -> T {
        cb(lock!(self.drawing_info).borrow_mut())
    }
}
