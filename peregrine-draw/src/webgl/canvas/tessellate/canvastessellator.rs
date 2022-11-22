use peregrine_toolkit::error::Error;

use crate::{Message, shape::core::texture::CanvasTextureArea};

#[cfg_attr(debug_assertions,derive(Debug))]
pub(crate) struct FlatBoundary {
    origin: Option<(u32,u32)>,
    size: Option<(u32,u32)>,
    padding: (u32,u32)
}

fn unpack<T: Clone>(data: &Option<T>) -> Result<T,Message> {
    data.as_ref().cloned().ok_or_else(|| Message::CodeInvariantFailed("texture packing failure, t origin".to_string()))
}

impl FlatBoundary {
    pub(crate) fn new() -> FlatBoundary {
        FlatBoundary { origin: None, size: None, padding: (0,0) }
    }

    fn size_without_padding(&self) -> Result<(u32,u32),Error> {
        self.size.ok_or_else(|| Error::fatal("texture get size unset"))
    }

    pub(crate) fn size_with_padding(&self) -> Result<(u32,u32),Error> {
        let size = self.size_without_padding()?;
        Ok((size.0+self.padding.0,size.1+self.padding.1))
    }

    pub(crate) fn origin(&self) -> Result<(u32,u32),Error> {
        self.origin.ok_or_else(|| Error::fatal("texture get size unset"))
    }

    pub(crate) fn update_padded_size(&mut self, size: (u32,u32)) {
        self.size = Some((size.0-self.padding.0,size.1-self.padding.1));
    }

    pub(crate) fn set_size(&mut self, size: (u32,u32), padding: (u32,u32)) {
        self.size = Some(size);
        self.padding = padding;
    }

    pub(crate) fn set_origin(&mut self, text: (u32,u32)) {
        self.origin = Some(text);
    }

    fn pad(&self, v: (u32,u32)) -> (u32,u32) {
        (v.0+self.padding.0,v.1+self.padding.1)
    }

    pub(crate) fn drawn_area(&self) -> Result<CanvasTextureArea,Message> {
        Ok(CanvasTextureArea::new(
            self.pad(unpack(&self.origin)?),
            unpack(&self.size)?
        ))
    }
}

pub(crate) struct CanvasTessellationPrepare {
    origin: Vec<(u32,u32)>,
    sizes: Vec<(u32,u32)>
}

impl CanvasTessellationPrepare {
    pub(crate) fn new() -> CanvasTessellationPrepare {
        CanvasTessellationPrepare { origin: vec![], sizes: vec![] }
    }

    pub(crate) fn add_size(&mut self, item: (u32,u32)) {
        self.sizes.push(item);
    }

    pub(crate) fn add_origin(&mut self, item: (u32,u32)) {
        self.origin.push(item);
    }

    pub fn expand_to_canvas(&mut self, x: Option<u32>, y: Option<u32>) {
        if let Some(x) = x {
            for size in &mut self.sizes { size.0 = x; }
        }
        if let Some(y) = y {
            for size in &mut self.sizes { size.1 = y; }
        }
    }

    pub(crate) fn origin(&self) -> &[(u32,u32)] { &self.origin }
    pub(crate) fn size(&self) -> &[(u32,u32)] { &self.sizes }
}
