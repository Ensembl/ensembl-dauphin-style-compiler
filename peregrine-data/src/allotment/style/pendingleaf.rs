use std::sync::Arc;

use crate::{allotment::{transformers::drawinginfo::DrawingInfo, boxes::boxtraits::Transformable}};
use super::allotmentname::AllotmentName;

pub struct PendingLeaf {
    name: AllotmentName,
    drawing_info: DrawingInfo,
    transformable: Option<Arc<dyn Transformable>>
}

impl PendingLeaf {
    pub fn new(name: &AllotmentName) -> PendingLeaf {
        PendingLeaf {
            name: name.clone(),
            drawing_info: DrawingInfo::new(),
            transformable: None
        }
    }

    pub(crate) fn set_transformable(&mut self, xformable: Arc<dyn Transformable>) {
        self.transformable = Some(xformable);
    }

    /* only call after set_transformable has been called! (via make_transformable) */
    pub fn transformable(&self) -> &Arc<dyn Transformable> { self.transformable.as_ref().unwrap() }
    
    pub fn name(&self) -> &AllotmentName {&self.name }
    pub fn drawing_info(&self) -> &DrawingInfo { &self.drawing_info }
    pub fn drawing_info_mut(&mut self) -> &mut DrawingInfo { &mut self.drawing_info }
}
