use crate::webgl::{ ProtoProcess, UniformHandle, Program };
use peregrine_data::DirectColour;
use super::super::util::arrayutil::{ scale_colour };
use crate::util::message::Message;

#[derive(Clone)]
pub struct SpotProgram {
    uniform: UniformHandle
}

impl SpotProgram {
    pub(crate) fn new(program: &Program) -> Result<SpotProgram,Message> {
        Ok(SpotProgram {
            uniform: program.get_uniform_handle("uColour")?
        })
    }
}

#[derive(Clone)]
pub struct SpotColourDraw {
    colour: DirectColour,
    variety: SpotProgram
}

impl SpotColourDraw {
    pub(crate) fn new(colour: &DirectColour, variety: &SpotProgram) -> Result<SpotColourDraw,Message> {
        Ok(SpotColourDraw { colour: colour.clone(), variety: variety.clone() })
    }

    pub(crate) fn spot(&self, process: &mut ProtoProcess) -> Result<(),Message> {
        process.set_uniform(&self.variety.uniform,vec![
            scale_colour(self.colour.0),scale_colour(self.colour.1),scale_colour(self.colour.2)])?;
        Ok(())
    }
}
