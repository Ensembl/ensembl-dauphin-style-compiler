use peregrine_toolkit::error::Error;

use crate::webgl::CanvasInUse;

use super::canvasitem::{CanvasItemArea, CanvasItemSize};

#[cfg_attr(debug_assertions,derive(Debug))]
pub(crate) struct CanvasItemAreaBuilder {
    origin: Option<(u32,u32)>,
    size: CanvasItemSize,
    area: Option<CanvasItemArea>
}

impl CanvasItemAreaBuilder {
    pub(crate) fn new(size: CanvasItemSize) -> CanvasItemAreaBuilder {
        CanvasItemAreaBuilder {
            origin: None,
            size,
            area: None
        }
    }

    pub(crate) fn size(&self) -> (u32,u32) { self.size.padded_size() }
    pub(crate) fn area(&self) -> CanvasItemArea { self.area.clone().unwrap() }

    pub(crate) fn origin(&self) -> Result<(u32,u32),Error> {
        self.origin.ok_or_else(|| Error::fatal("texture get size unset"))
    }

    pub(crate) fn set_origin(&mut self, text: (u32,u32)) {
        self.origin = Some(text);
    }

    pub(crate) fn build(&mut self, x: Option<u32>, y: Option<u32>) -> Result<(),Error> {
        self.size.extend(x,y);
        self.area = Some(CanvasItemArea::new(
            self.size.pad_origin(self.origin.unwrap()),
            self.size.unpadded_size()
        ));
        Ok(())
    }
}
