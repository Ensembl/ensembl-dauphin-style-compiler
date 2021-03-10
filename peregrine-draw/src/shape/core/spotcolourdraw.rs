use crate::webgl::{ ProtoProcess, UniformHandle, Program };
use peregrine_data::DirectColour;
use super::super::util::arrayutil::{ scale_colour };

#[derive(Clone)]
pub struct SpotProgram {
    uniform: UniformHandle
}

impl SpotProgram {
    pub(crate) fn new(program: &Program) -> anyhow::Result<SpotProgram> {
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
    pub(crate) fn new(_process: &ProtoProcess, colour: &DirectColour, variety: &SpotProgram) -> anyhow::Result<SpotColourDraw> {
        Ok(SpotColourDraw { colour: colour.clone(), variety: variety.clone() })
    }

    pub(crate) fn spot(&self, process: &mut ProtoProcess) -> anyhow::Result<()> {
        process.set_uniform(&self.variety.uniform,vec![
            scale_colour(self.colour.0),scale_colour(self.colour.1),scale_colour(self.colour.2)])?;
        Ok(())
    }
}
