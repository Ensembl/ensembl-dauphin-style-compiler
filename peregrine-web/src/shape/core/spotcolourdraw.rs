use anyhow::bail;
use crate::webgl::{ AttribHandle, ProtoProcess, AccumulatorCampaign, UniformHandle, Program };
use peregrine_core::DirectColour;
use super::super::layers::arrayutil::{ scale_colour };

#[derive(Clone)]
pub struct SpotColourDrawVariety {
    uniform: UniformHandle
}

impl SpotColourDrawVariety {
    pub(crate) fn new(program: &Program) -> anyhow::Result<SpotColourDrawVariety> {
        Ok(SpotColourDrawVariety {
            uniform: program.get_uniform_handle("uColour")?
        })
    }
}

#[derive(Clone)]
pub struct SpotColourDraw {
    colour: DirectColour,
    variety: SpotColourDrawVariety
}

impl SpotColourDraw {
    pub(crate) fn new(process: &ProtoProcess, colour: &DirectColour, variety: &SpotColourDrawVariety) -> anyhow::Result<SpotColourDraw> {
        Ok(SpotColourDraw { colour: colour.clone(), variety: variety.clone() })
    }

    pub(crate) fn run(&self, process: &mut ProtoProcess) -> anyhow::Result<()> {
        process.set_uniform(&self.variety.uniform,vec![
            scale_colour(self.colour.0),scale_colour(self.colour.1),scale_colour(self.colour.2)])?;
        Ok(())
    }
}
