use crate::webgl::{ AttribHandle, ProtoProcess, ProcessStanzaAddable, Program };
use crate::webgl::{ FlatId };
use crate::webgl::TextureBindery;
use crate::util::message::Message;

pub struct CanvasTextureAreas {
    texture_origin: (u32,u32),
    mask_origin: (u32,u32),
    size: (u32,u32)
}

impl CanvasTextureAreas {
    pub(crate) fn new(texture_origin: (u32,u32), mask_origin: (u32,u32), size: (u32,u32)) -> CanvasTextureAreas {
        CanvasTextureAreas { texture_origin, mask_origin, size }
    }

    pub(crate) fn texture_origin(&self) -> (u32,u32) { self.texture_origin }
    pub(crate) fn mask_origin(&self) -> (u32,u32) { self.mask_origin }
    pub(crate) fn size(&self) -> (u32,u32) { self.size }
}

#[derive(Clone)]
pub struct TextureProgram {
    texture: AttribHandle,
    mask: AttribHandle
}

impl TextureProgram {
    pub(crate) fn new(program: &Program) -> Result<TextureProgram,Message> {
        Ok(TextureProgram {
            texture: program.get_attrib_handle("aTextureCoord")?,
            mask: program.get_attrib_handle("aMaskCoord")?,
        })
    }
}

#[derive(Clone)]
pub struct TextureDraw(TextureProgram);

// TODO structify
// TODO to array utils

impl TextureDraw {
    pub(crate) fn new(variety: &TextureProgram) -> Result<TextureDraw,Message> {
        Ok(TextureDraw(variety.clone()))
    }

    fn add_rectangle_one(&self, addable: &mut dyn ProcessStanzaAddable, attrib: &AttribHandle, dims: &mut dyn Iterator<Item=((u32,u32),(u32,u32))>) -> Result<(),Message> {
        let mut data = vec![];
        for (origin,size) in dims {
            data.push(origin.0 as f64); data.push(origin.1 as f64); // (min,min)
            data.push(origin.0 as f64); data.push((origin.1+size.1) as f64); // (min,max)
            data.push((origin.0+size.0) as f64); data.push(origin.1 as f64); // (max,min)
            data.push((origin.0+size.0) as f64); data.push((origin.1+size.1) as f64); // (max,max)
        }
        addable.add_n(attrib,data,2)?;
        Ok(())
    }

    pub(crate) fn add_rectangle(&self, process: &mut ProtoProcess, addable: &mut dyn ProcessStanzaAddable, bindery: &TextureBindery, canvas: &FlatId, dims: &[CanvasTextureAreas]) -> Result<(),Message> {
        self.add_rectangle_one(addable,&self.0.texture,&mut dims.iter().map(|x| (x.texture_origin(),x.size())))?;
        self.add_rectangle_one(addable,&self.0.mask,&mut dims.iter().map(|x| (x.mask_origin(),x.size())))?;
        Ok(())
    }
}
