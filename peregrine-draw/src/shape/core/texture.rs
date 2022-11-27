use peregrine_toolkit::error::Error;
use crate::shape::layers::patina::{ PatinaProcessName, Freedom};
use crate::webgl::canvas::composition::canvasitem::CanvasItemArea;
use crate::webgl::canvas::htmlcanvas::canvasinuse::CanvasInUse;
use crate::webgl::{ AttribHandle, ProcessStanzaAddable, ProgramBuilder, UniformHandle, ProcessBuilder };

#[derive(Clone)]
pub struct TextureProgram {
    texture: AttribHandle,
    freedom: Option<UniformHandle>
}

/* TODO carry on dismanyling / pushing out Patina half. Make this more like TriangleAdder, its
 * geometry counterpart, by switching from ProgramBuilder to Process builder. (Note, both will)
 * ultimately need lookups genericised to program level to avoid too many lookups.
 */

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
    pub(crate) fn new(builder: &mut ProcessBuilder, freedom: &Freedom) -> Result<TextureDraw,Error> {
        let program = TextureProgram::new(builder.program_builder())?;
        let process_name = builder.patina_name();
        let canvas = process_name.canvas_name().cloned();
        if let Some(canvas) = &canvas {
            builder.set_texture("uSampler",canvas)?;
        }
        let x = freedom.as_gl();
        if let Some(handle) = &program.freedom {
            builder.set_uniform(handle,vec![x.0,x.1])?;
        }        
        Ok(TextureDraw(program.clone(),freedom.is_free()))
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

    pub(crate) fn add_rectangle(&self, addable: &mut dyn ProcessStanzaAddable, canvas: &CanvasInUse, dims: &[CanvasItemArea], freedom: &Freedom) -> Result<(),Error> {
        let size = canvas.retrieve(|flat| { flat.size().clone() });
        let mut texture_data = dims.iter()
            .map(|x| (x.origin(),x.size()));
        self.add_rectangle_one(addable,&self.0.texture,&mut texture_data,&size,&freedom)?;
        Ok(())
    }
}

pub(crate) fn xxx_texture_name(flat_id: &CanvasInUse, freedom: &Freedom) -> PatinaProcessName {
    match freedom {
        Freedom::None => PatinaProcessName::Texture(flat_id.clone()),
        Freedom::Horizontal => PatinaProcessName::FreeTexture(flat_id.clone(),Freedom::Horizontal),
        Freedom::Vertical => PatinaProcessName::FreeTexture(flat_id.clone(),Freedom::Vertical),
    }
}
