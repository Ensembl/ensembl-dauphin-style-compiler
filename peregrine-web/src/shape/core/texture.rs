use crate::webgl::{ UniformHandle, ProtoProcess, ProcessStanzaAddable, Program };
use peregrine_core::DirectColour;
use super::arrayutil::scale_colour;
use super::super::canvas::weave::CanvasRequestId;

#[derive(Clone)]
pub struct TextureProgram {
    sampler: UniformHandle
}

impl TextureProgram {
    pub(crate) fn new(program: &Program) -> anyhow::Result<TextureProgram> {
        Ok(TextureProgram {
            sampler: program.get_uniform_handle("uSampler")?
        })
    }
}

#[derive(Clone)]
pub struct TextureDraw(TextureProgram);

impl TextureDraw {
    pub(crate) fn new(_process: &ProtoProcess, variety: &TextureProgram, canvas_id: &CanvasRequestId) -> anyhow::Result<TextureDraw> {
        
        Ok(TextureDraw(variety.clone()))
    }

    /*
    pub(crate) fn direct(&self, addable: &mut dyn ProcessStanzaAddable, colours: &[DirectColour], vertexes: usize) -> anyhow::Result<()> {
        let mut codes = vec![];
        for c in colours {
            for _ in 0..vertexes {
                codes.push(scale_colour(c.0));
                codes.push(scale_colour(c.1));
                codes.push(scale_colour(c.2));
            }
        }
        addable.add_n(&self.0.colour,codes)?;
        Ok(())
    }
    */
}
