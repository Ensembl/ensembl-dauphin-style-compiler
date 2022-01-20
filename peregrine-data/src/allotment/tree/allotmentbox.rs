use std::sync::{Arc, Mutex};

use peregrine_toolkit::lock;

use crate::{AllotmentMetadata, allotment::core::{allotmentrequest::AllotmentRequestImpl, allotment::Transformer, arbitrator::DelayedValue}};

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct AllotmentBoxBuilder {
    padding_top: i64,
    padding_bottom: i64,
    min_height: Option<i64>,
    children: Vec<AllotmentBox>,
    natural_height: i64
}

impl AllotmentBoxBuilder {
    pub fn new(metadata: &AllotmentMetadata, natural_height: i64) -> AllotmentBoxBuilder {
        AllotmentBoxBuilder {
            padding_top: metadata.get_i64("padding-top").unwrap_or(0),
            padding_bottom: metadata.get_i64("padding-bottom").unwrap_or(0),
            min_height: metadata.get_i64("min-height"),
            natural_height,
            children: vec![]
        }
    }

    pub fn empty() -> AllotmentBoxBuilder {
        AllotmentBoxBuilder {
            padding_top: 0,
            padding_bottom: 0,
            min_height: None,
            natural_height: 0,
            children: vec![]
        }
    }

    fn unpadded_height(&self) -> i64 {
        self.natural_height.max(self.min_height.unwrap_or(0))
    }

    /* don't make visible except to AllotmentBox */
    fn padded_height(&self) -> i64 {
        self.natural_height.max(self.min_height.unwrap_or(0))
    }

    /* don't make visible except to AllotmentBox */
    fn set_root(&self, container_offset: i64) {
        for child in &self.children {
            child.set_root(container_offset);
        }
    }

    pub fn append(&mut self, child: AllotmentBox) {
        child.set_container_offset(self.natural_height);
        self.natural_height += child.total_height();
        self.children.push(child);
    }

    pub fn overlay(&mut self, child: AllotmentBox) {
        child.set_container_offset(0);
        self.natural_height = self.natural_height.max(child.total_height());
        self.children.push(child);
    }

    pub fn append_all(&mut self, mut children: Vec<AllotmentBox>) {
        for b in children.drain(..) {
            self.append(b);
        }
    }

    pub fn overlay_all(&mut self, mut children: Vec<AllotmentBox>) {
        for b in children.drain(..) {
            self.overlay(b);
        }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub struct AllotmentBox {
    offset_from_container: Arc<Mutex<i64>>,
    offset_from_root: Arc<Mutex<i64>>,
    allot_box: Arc<AllotmentBoxBuilder>
}

impl AllotmentBox {
    pub fn new(builder: AllotmentBoxBuilder) -> AllotmentBox {
        AllotmentBox {
            offset_from_container: Arc::new(Mutex::new(0)),
            offset_from_root: Arc::new(Mutex::new(0)),
            allot_box: Arc::new(builder)
        }
    }

    fn set_container_offset(&self, offset: i64) {
        *lock!(self.offset_from_container) = offset;
    }

    pub fn set_root(&self, container_offset: i64) {
        let offset_from_root= *lock!(self.offset_from_container) + container_offset;
        *lock!(self.offset_from_root) = offset_from_root;
        self.allot_box.set_root(offset_from_root);
    }

    pub fn top(&self) -> i64 { *lock!(self.offset_from_root) }
    pub fn total_height(&self) -> i64 { self.allot_box.padded_height() }
    pub fn bottom(&self) -> i64 { self.top() + self.total_height() }

    pub fn top_delayed(&self) -> DelayedValue {
        DelayedValue::new(&self.offset_from_root,|x| x)
    }

    pub fn bottom_delayed(&self) -> DelayedValue {
        let height = self.total_height();
        DelayedValue::new(&self.offset_from_root,move |x| x + height)
    }

    pub fn draw_top(&self) -> i64 { self.top() + self.allot_box.padding_top }
    pub fn draw_bottom(&self) -> i64 { self.draw_top() + self.allot_box.unpadded_height() }
}
