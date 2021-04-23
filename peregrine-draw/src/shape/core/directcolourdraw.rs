use crate::webgl::{ AttribHandle, ProcessStanzaAddable, ProgramBuilder };
use peregrine_data::DirectColour;
use super::super::util::arrayutil::scale_colour;
use crate::util::message::Message;

#[derive(Clone)]
pub struct DirectProgram {
    colour: AttribHandle
}

impl DirectProgram {
    pub(crate) fn new(builder: &ProgramBuilder) -> Result<DirectProgram,Message> {
        Ok(DirectProgram {
            colour: builder.get_attrib_handle("aVertexColour")?
        })
    }
}

#[derive(Clone)]
pub struct DirectColourDraw(DirectProgram);

impl DirectColourDraw {
    pub(crate) fn new(variety: &DirectProgram) -> Result<DirectColourDraw,Message> {
        Ok(DirectColourDraw(variety.clone()))
    }

    pub(crate) fn direct(&self, addable: &mut dyn ProcessStanzaAddable, colours: &[DirectColour], vertexes: usize) -> Result<(),Message> {
        let mut codes = vec![];
        for c in colours {
            for _ in 0..vertexes {
                codes.push(scale_colour(c.0));
                codes.push(scale_colour(c.1));
                codes.push(scale_colour(c.2));
            }
        }
        addable.add_n(&self.0.colour,codes,3)?;
        Ok(())
    }
}
