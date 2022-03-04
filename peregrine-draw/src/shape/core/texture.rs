use crate::shape::layers::patina::{PatinaProcess, PatinaProcessName, PatinaAdder, PatinaYielder};
use crate::webgl::{ AttribHandle, ProcessStanzaAddable, ProgramBuilder };
use crate::webgl::{ FlatId };
use crate::util::message::Message;
use crate::webgl::canvas::flatstore::FlatStore;

fn scale(pair: (u32,u32), factor: f64) -> (u32,u32) {
    (
        (pair.0 as f64 * factor) as u32,
        (pair.1 as f64 * factor) as u32
    )
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct CanvasTextureArea {
    texture_origin: (u32,u32),
    mask_origin: (u32,u32),
    size: (u32,u32)
}

impl CanvasTextureArea {
    pub(crate) fn new(texture_origin: (u32,u32), mask_origin: (u32,u32), size: (u32,u32)) -> CanvasTextureArea {
        CanvasTextureArea { texture_origin, mask_origin, size }
    }

    pub(crate) fn scale(&self, factor: f64) -> CanvasTextureArea {
        CanvasTextureArea {
            texture_origin: scale(self.texture_origin,factor),
            mask_origin: scale(self.mask_origin,factor),
            size: scale(self.size,factor)
        }
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
    pub(crate) fn new(builder: &ProgramBuilder) -> Result<TextureProgram,Message> {
        Ok(TextureProgram {
            texture: builder.get_attrib_handle("aTextureCoord")?,
            mask: builder.get_attrib_handle("aMaskCoord")?,
        })
    }
}

#[derive(Clone)]
pub struct TextureDraw(TextureProgram,bool);

// TODO structify
// TODO to array utils

fn push(data: &mut Vec<f32>,x: u32, y: u32, size: &(u32,u32)) {
    data.push((x as f32)/(size.0 as f32));
    data.push((y as f32)/(size.1 as f32));
}

impl TextureDraw {
    pub(crate) fn new(variety: &TextureProgram, free: bool) -> Result<TextureDraw,Message> {
        Ok(TextureDraw(variety.clone(),free))
    }

    fn add_rectangle_one(&self, addable: &mut dyn ProcessStanzaAddable, attrib: &AttribHandle, dims: &mut dyn Iterator<Item=((u32,u32),(u32,u32))>, csize: &(u32,u32)) -> Result<(),Message> {
        let mut data = vec![];
        for (origin,size) in dims {
            let size = if self.1 { (0,0) } else { size };
            push(&mut data, origin.0,origin.1,&csize);
            push(&mut data, origin.0,origin.1+size.1,&csize);
            push(&mut data, origin.0+size.0,origin.1,&csize);
            push(&mut data, origin.0+size.0,origin.1+size.1,&csize);
        }
        addable.add_n(attrib,data,2)?;
        Ok(())
    }

    pub(crate) fn add_rectangle(&self, addable: &mut dyn ProcessStanzaAddable, canvas: &FlatId, dims: &[CanvasTextureArea], flat_store: &FlatStore) -> Result<(),Message> {
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

pub(crate) struct TextureYielder {
    patina_process_name: PatinaProcessName,
    texture: Option<TextureDraw>
}

impl TextureYielder {
    pub(crate) fn new(flat_id: &FlatId, free: bool) -> TextureYielder {
        let patina_process_name = if free { PatinaProcessName::FreeTexture(flat_id.clone()) } else { PatinaProcessName::Texture(flat_id.clone()) };
        TextureYielder { 
            texture: None,
            patina_process_name
        }
    }

    pub(crate) fn draw(&self) -> Result<&TextureDraw,Message> {
        self.texture.as_ref().ok_or_else(|| Message::CodeInvariantFailed(format!("using accessor without setting")))
    }
}

impl PatinaYielder for TextureYielder {
    fn name(&self) -> &PatinaProcessName { &self.patina_process_name }

    fn make(&mut self, builder: &ProgramBuilder) -> Result<PatinaAdder,Message> {
        Ok(PatinaAdder::Texture(TextureProgram::new(builder)?))
    }
    
    fn set(&mut self, program: &PatinaProcess) -> Result<(),Message> {
        self.texture = Some(match program {
            PatinaProcess::FreeTexture(t) => t,
            PatinaProcess::Texture(t) => t,
            _ => { Err(Message::CodeInvariantFailed(format!("mismatched program: texture")))? }
        }.clone());
        Ok(())
    }
}
