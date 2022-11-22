use std::sync::{Arc, Mutex};

use peregrine_toolkit::{error::Error, lock, log};
use crate::{shape::core::{texture::CanvasTextureArea}};

#[cfg_attr(debug_assertions,derive(Debug))]
struct FlatBoundaryImpl {
    origin: Option<(u32,u32)>,
    size: (u32,u32),
    padding: (u32,u32)

}

impl FlatBoundaryImpl {
    fn pad(&self, v: (u32,u32)) -> (u32,u32) {
        (v.0+self.padding.0,v.1+self.padding.1)
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub(crate) struct FlatBoundary(Arc<Mutex<FlatBoundaryImpl>>);

fn unpack<T: Clone>(data: &Option<T>) -> Result<T,Error> {
    data.as_ref().cloned().ok_or_else(|| Error::fatal("texture packing failure, t origin"))
}

impl FlatBoundary {
    pub(crate) fn new(size: (u32,u32), padding: (u32,u32)) -> FlatBoundary {
        FlatBoundary(Arc::new(Mutex::new(FlatBoundaryImpl { origin: None, size, padding })))
    }

    pub(crate) fn size_with_padding(&self) -> Result<(u32,u32),Error> {
        let state = lock!(self.0);
        Ok((state.size.0+state.padding.0,state.size.1+state.padding.1))
    }

    pub(crate) fn origin(&self) -> Result<(u32,u32),Error> {
        lock!(self.0).origin.ok_or_else(|| Error::fatal("texture get size unset"))
    }

    pub(crate) fn set_origin(&mut self, text: (u32,u32)) {
        lock!(self.0).origin = Some(text);
    }

    pub(crate) fn drawn_area(&self) -> Result<CanvasTextureArea,Error> {
        let state = lock!(self.0);
        Ok(CanvasTextureArea::new(
            state.pad(unpack(&state.origin)?),
            state.size
        ))
    }

    pub fn expand_to_canvas(&mut self, x: Option<u32>, y: Option<u32>) {
        let mut state = lock!(self.0);
        if let Some(x) = x { state.size.0 = x-state.padding.0; }
        if let Some(y) = y { state.size.1 = y-state.padding.1; }
        log!("expanded to {:?}",state.size);
    }
}

pub(crate) struct CanvasTessellationPrepare {
    items: Vec<FlatBoundary>,
    with_origin: usize
}

impl CanvasTessellationPrepare {
    pub(crate) fn new() -> CanvasTessellationPrepare {
        CanvasTessellationPrepare { items: vec![], with_origin: 0 }
    }

    pub(crate) fn add(&mut self, item: FlatBoundary) -> Result<(),Error> {
        self.items.push(item);
        Ok(())
    }

    pub(crate) fn add_origin(&mut self, origin: (u32,u32)) {
        self.items[self.with_origin].set_origin(origin);
        self.with_origin += 1;
    }

    pub(crate) fn bump(&mut self, len: usize) {
        self.with_origin += len;
    }

    pub fn expand_to_canvas(&mut self, x: Option<u32>, y: Option<u32>) {
        for item in &mut self.items {
            item.expand_to_canvas(x,y);
        }
    }

    pub(crate) fn items(&self) -> &[FlatBoundary] { &self.items }
    pub(crate) fn items_mut(&mut self) -> &mut Vec<FlatBoundary> { &mut self.items }
}
