use anyhow::bail;
use crate::webgl::{ AttribHandle, ProcessBuilder, AccumulatorCampaign, UniformHandle };
use peregrine_core::DirectColour;
use super::arrayutil::{ scale_colour };

#[derive(Clone)]
pub struct SpotColourDraw {
    colour: DirectColour,
    uniform: UniformHandle
}

impl SpotColourDraw {
    pub(crate) fn new(process: &ProcessBuilder, colour: &DirectColour) -> anyhow::Result<SpotColourDraw> {
        let uniform = process.get_uniform_handle("uColour")?;
        Ok(SpotColourDraw { colour: colour.clone(), uniform })
    }

    pub(crate) fn run(&self, process: &mut ProcessBuilder) -> anyhow::Result<()> {
        process.set_uniform(&self.uniform,vec![
            scale_colour(self.colour.0),scale_colour(self.colour.1),scale_colour(self.colour.2)])?;
        Ok(())
    }
}
