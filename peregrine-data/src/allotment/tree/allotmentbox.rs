use std::sync::{Arc, Mutex};

use peregrine_toolkit::lock;

use crate::{AllotmentMetadata, allotment::core::{allotmentrequest::AllotmentRequestImpl, allotment::Transformer, arbitrator::DelayedValue}};

pub struct AllotmentBoxBuilder {
    padding_top: i64,
    padding_bottom: i64,
    min_height: Option<i64>,
    children: Vec<AllotmentBox>,
    natural_height: i64,
    self_indent: Option<DelayedValue>
}

impl AllotmentBoxBuilder {
    pub fn new(metadata: &AllotmentMetadata, natural_height: i64) -> AllotmentBoxBuilder {
        AllotmentBoxBuilder {
            padding_top: metadata.get_i64("padding-top").unwrap_or(0),
            padding_bottom: metadata.get_i64("padding-bottom").unwrap_or(0),
            min_height: metadata.get_i64("min-height"),
            natural_height,
            children: vec![],
            self_indent: None
        }
    }

    pub fn empty() -> AllotmentBoxBuilder {
        AllotmentBoxBuilder {
            padding_top: 0,
            padding_bottom: 0,
            min_height: None,
            natural_height: 0,
            children: vec![],
            self_indent: None
        }
    }

    pub fn set_self_indent(&mut self, indent: Option<&DelayedValue>) {
        self.self_indent = indent.cloned();
    }

    fn unpadded_height(&self) -> i64 {
        self.natural_height.max(self.min_height.unwrap_or(0))
    }

    /* don't make visible except to AllotmentBox */
    fn padded_height(&self) -> i64 {
        self.natural_height.max(self.min_height.unwrap_or(0))
    }

    /* don't make visible except to AllotmentBox */
    fn apply_root(&self, container_offset: i64) {
        for child in &self.children {
            child.apply_root(container_offset);
        }
    }

    /* don't make visible except to AllotmentBox */
    fn apply_indent(&self, indent: i64) {
        for child in &self.children {
            child.apply_indent(indent);
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

#[derive(Clone)]
pub struct AllotmentBox {
    indent: DelayedValue,
    offset_from_container: DelayedValue,
    offset_from_root: DelayedValue,
    allot_box: Arc<AllotmentBoxBuilder>,
}

impl AllotmentBox {
    pub fn new(builder: AllotmentBoxBuilder) -> AllotmentBox {
        AllotmentBox {
            offset_from_container: DelayedValue::fixed(0),
            offset_from_root: DelayedValue::fixed(0),
            indent: DelayedValue::fixed(0),
            allot_box: Arc::new(builder)
        }
    }

    fn set_container_offset(&self, offset: i64) {
        self.offset_from_container.set_value(offset);
    }

    fn apply_root(&self, container_offset: i64) {
        let offset_from_root= self.offset_from_container.value() + container_offset;
        self.offset_from_root.set_value(offset_from_root);
        self.allot_box.apply_root(offset_from_root);
    }

    fn apply_indent(&self, indent: i64) {
        let indent = self.allot_box.self_indent.as_ref().map(|x| x.value()).unwrap_or(indent);
        self.indent.set_value(indent);
        self.allot_box.apply_indent(indent);
    }

    pub fn set_root(&self, container_offset: i64, indent: i64) {
        self.apply_root(container_offset);
        self.apply_indent(indent);
    }

    pub fn top(&self) -> i64 { self.offset_from_root.value() }
    pub fn total_height(&self) -> i64 { self.allot_box.padded_height() }
    pub fn bottom(&self) -> i64 { self.top() + self.total_height() }

    pub fn top_delayed(&self) -> DelayedValue { self.offset_from_root.clone() }

    pub fn bottom_delayed(&self) -> DelayedValue {
        let height = self.total_height();
        DelayedValue::derived(&self.offset_from_root,move |x| x + height)
    }

    pub fn draw_top(&self) -> i64 { self.top() + self.allot_box.padding_top }
    pub fn draw_bottom(&self) -> i64 { self.draw_top() + self.allot_box.unpadded_height() }

    pub fn indent_delayed(&self) -> DelayedValue { self.indent.clone() }
    pub fn indent(&self) -> i64 { self.indent.value() }
}
