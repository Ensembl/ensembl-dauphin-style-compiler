use std::sync::{Arc, Mutex};
use peregrine_toolkit::{error::Error, lock};
use crate::{shape::core::{texture::CanvasTextureArea}};

#[cfg_attr(debug_assertions,derive(Debug))]
pub(crate) struct FlatBoundary {
    origin: Option<(u32,u32)>,
    size: (u32,u32),
    padding: (u32,u32),
    area: Option<CanvasTextureArea>
}

pub(crate) struct CanvasItemSize((u32,u32),(u32,u32));

impl CanvasItemSize {
    pub(crate) fn new(size: (u32,u32), padding: (u32,u32)) -> CanvasItemSize {
        CanvasItemSize(size,padding)
    }
}

fn unpack<T: Clone>(data: &Option<T>) -> Result<T,Error> {
    data.as_ref().cloned().ok_or_else(|| Error::fatal("texture packing failure, t origin"))
}

impl FlatBoundary {
    pub(crate) fn new(size: CanvasItemSize) -> FlatBoundary {
        FlatBoundary {
            origin: None,
            size: size.0,
            padding: size.1,
            area: None
        }
    }

    fn pad(&self, v: (u32,u32)) -> (u32,u32) {
        (v.0+self.padding.0,v.1+self.padding.1)
    }

    pub(crate) fn size(&self) -> Result<(u32,u32),Error> {
        Ok((self.size.0+self.padding.0,self.size.1+self.padding.1))
    }

    pub(crate) fn area(&self) -> CanvasTextureArea { self.area.clone().unwrap() }

    pub(crate) fn origin(&self) -> Result<(u32,u32),Error> {
        self.origin.ok_or_else(|| Error::fatal("texture get size unset"))
    }

    pub(crate) fn set_origin(&mut self, text: (u32,u32)) {
        self.origin = Some(text);
    }

    pub fn build(&mut self, x: Option<u32>, y: Option<u32>) -> Result<(),Error> {
        if let Some(x) = x { self.size.0 = x-self.padding.0; }
        if let Some(y) = y { self.size.1 = y-self.padding.1; }
        self.area = Some(CanvasTextureArea::new(
            self.pad(unpack(&self.origin)?),
            self.size
        ));
        Ok(())
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct CanvasLocationSource(Arc<Mutex<Option<CanvasTextureArea>>>);

impl CanvasLocationSource {
    pub(crate) fn new() -> CanvasLocationSource {
        CanvasLocationSource(Arc::new(Mutex::new(None)))
    }

    pub(crate) fn set(&self, area: CanvasTextureArea) { *lock!(self.0) = Some(area); }

    pub(crate) fn get(&self) -> Result<CanvasTextureArea,Error> {
        lock!(self.0).as_ref().map(|x| x.clone()).ok_or_else(|| Error::fatal("source not ready"))
    }
}
