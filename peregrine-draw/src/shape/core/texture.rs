use peregrine_toolkit::error::Error;

use crate::shape::layers::patina::{PatinaProcess, PatinaProcessName, PatinaAdder, PatinaYielder, Freedom};
use crate::webgl::{ AttribHandle, ProcessStanzaAddable, ProgramBuilder, UniformHandle, ProcessBuilder };
use crate::webgl::{ CanvasInUse };

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct CanvasTextureArea {
    origin: (u32,u32),
    size: (u32,u32)
}

impl CanvasTextureArea {
    pub(crate) fn new(origin: (u32,u32), size: (u32,u32)) -> CanvasTextureArea {
        CanvasTextureArea { origin, size }
    }

    pub(crate) fn origin(&self) -> (u32,u32) { self.origin }
    pub(crate) fn size(&self) -> (u32,u32) { self.size }
}

#[derive(Clone)]
pub struct TextureProgram {
    texture: AttribHandle,
    freedom: Option<UniformHandle>
}

impl TextureProgram {
    pub(crate) fn new(builder: &ProgramBuilder) -> Result<TextureProgram,Error> {
        Ok(TextureProgram {
            texture: builder.get_attrib_handle("aTextureCoord")?,
            freedom: builder.try_get_uniform_handle("uFreedom")
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
    pub(crate) fn new(variety: &TextureProgram, free: bool) -> Result<TextureDraw,Error> {
        Ok(TextureDraw(variety.clone(),free))
    }

    fn add_rectangle_one(&self, addable: &mut dyn ProcessStanzaAddable, attrib: &AttribHandle, dims: &mut dyn Iterator<Item=((u32,u32),(u32,u32))>, csize: &(u32,u32), freedom: &Freedom) -> Result<(),Error> {
        let mut data = vec![];
        if self.1 {
            let (fx,fy)= freedom.as_gl().into();
            let (fx,fy) = (fx as u32, fy as u32);
            for (origin,size) in dims {
                push(&mut data, origin.0,origin.1,&csize);
                push(&mut data, origin.0,origin.1+size.1*fx,&csize);
                push(&mut data, origin.0+size.0*fy,origin.1,&csize);
                push(&mut data, origin.0+size.0*fy,origin.1+size.1*fx,&csize);
            }
        } else {
            for (origin,size) in dims {
                push(&mut data, origin.0,origin.1,&csize);
                push(&mut data, origin.0,origin.1+size.1,&csize);
                push(&mut data, origin.0+size.0,origin.1,&csize);
                push(&mut data, origin.0+size.0,origin.1+size.1,&csize);
            }
        }
        addable.add_n(attrib,data,2)?;
        Ok(())
    }

    pub(crate) fn set_freedom(&self, process: &mut ProcessBuilder, freedom: &Freedom) {
        let x = freedom.as_gl();
        if let Some(handle) = &self.0.freedom {
            process.set_uniform(handle,vec![x.0,x.1]);
        }
    }

    pub(crate) fn add_rectangle(&self, addable: &mut dyn ProcessStanzaAddable, canvas: &CanvasInUse, dims: &[CanvasTextureArea], freedom: &Freedom) -> Result<(),Error> {
        let size = canvas.retrieve(|flat| { flat.size().clone() });
        let mut texture_data = dims.iter()
            .map(|x| (x.origin(),x.size()));
        self.add_rectangle_one(addable,&self.0.texture,&mut texture_data,&size,&freedom)?;
        Ok(())
    }
}

pub(crate) struct TextureYielder {
    patina_process_name: PatinaProcessName,
    texture: Option<TextureDraw>
}

impl TextureYielder {
    pub(crate) fn new(flat_id: &CanvasInUse, freedom: &Freedom) -> TextureYielder {
        let patina_process_name = match freedom {
            Freedom::None => PatinaProcessName::Texture(flat_id.clone()),
            Freedom::Horizontal => PatinaProcessName::FreeTexture(flat_id.clone(),Freedom::Horizontal),
            Freedom::Vertical => PatinaProcessName::FreeTexture(flat_id.clone(),Freedom::Vertical),
        };
        TextureYielder { 
            texture: None,
            patina_process_name
        }
    }

    pub(crate) fn draw(&self) -> Result<&TextureDraw,Error> {
        self.texture.as_ref().ok_or_else(|| Error::fatal("using accessor without setting"))
    }
}

impl PatinaYielder for TextureYielder {
    fn name(&self) -> &PatinaProcessName { &self.patina_process_name }

    fn make(&mut self, builder: &ProgramBuilder) -> Result<PatinaAdder,Error> {
        Ok(PatinaAdder::Texture(TextureProgram::new(builder)?))
    }
    
    fn set(&mut self, program: &PatinaProcess) -> Result<(),Error> {
        self.texture = Some(match program {
            PatinaProcess::FreeTexture(t) => t,
            PatinaProcess::Texture(t) => t,
            _ => { Err(Error::fatal("mismatched program: texture"))? }
        }.clone());
        Ok(())
    }
}
