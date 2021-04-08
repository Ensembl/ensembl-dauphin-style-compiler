use crate::webgl::{ AttribHandle, ProtoProcess, ProcessStanzaAddable, Program };
use crate::webgl::{ FlatId };
use crate::webgl::TextureBindery;
use crate::util::message::Message;
use crate::webgl::canvas::flatstore::FlatStore;

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

fn push(data: &mut Vec<f64>,x: u32, y: u32, size: &(u32,u32)) {
    data.push((x as f64)/(size.0 as f64)); data.push((y as f64)/(size.1 as f64));
}

impl TextureDraw {
    pub(crate) fn new(variety: &TextureProgram) -> Result<TextureDraw,Message> {
        Ok(TextureDraw(variety.clone()))
    }

    fn add_rectangle_one(&self, addable: &mut dyn ProcessStanzaAddable, attrib: &AttribHandle, dims: &mut dyn Iterator<Item=((u32,u32),(u32,u32))>, csize: &(u32,u32)) -> Result<(),Message> {
        let mut data = vec![];
        for (origin,size) in dims {
            push(&mut data, origin.0,origin.1,&csize);
            push(&mut data, origin.0,origin.1+size.1,&csize);
            push(&mut data, origin.0+size.0,origin.1,&csize);
            push(&mut data, origin.0+size.0,origin.1+size.1,&csize);
        }
        addable.add_n(attrib,data,2)?;
        Ok(())
    }

    pub(crate) fn add_rectangle(&self, process: &mut ProtoProcess, addable: &mut dyn ProcessStanzaAddable, bindery: &TextureBindery, canvas: &FlatId, dims: &[CanvasTextureAreas],flat_store: &FlatStore) -> Result<(),Message> {
        let size = flat_store.get(canvas)?.size();
        let mut texture_data = dims.iter()
            .map(|x| (x.texture_origin(),x.size()));
        let mut mask_data = dims.iter()
            .map(|x| (x.mask_origin(),x.size()));
        self.add_rectangle_one(addable,&self.0.texture,&mut texture_data,size)?;
        self.add_rectangle_one(addable,&self.0.mask,&mut mask_data,size)?;
        Ok(())
    }
}
