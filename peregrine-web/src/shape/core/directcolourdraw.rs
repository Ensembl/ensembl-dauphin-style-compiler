use crate::webgl::{ AttribHandle, ProtoProcess, AccumulatorCampaign, Program };
use peregrine_core::DirectColour;
use super::super::layers::arrayutil::scale_colour;

#[derive(Clone)]
pub struct DirectColourDrawVariety {
    colour: AttribHandle
}

impl DirectColourDrawVariety {
    pub(crate) fn new(program: &Program) -> anyhow::Result<DirectColourDrawVariety> {
        Ok(DirectColourDrawVariety {
            colour: program.get_attrib_handle("aVertexColour")?
        })
    }
}

#[derive(Clone)]
pub struct DirectColourDraw(DirectColourDrawVariety);

impl DirectColourDraw {
    pub(crate) fn new(process: &ProtoProcess, variety: &DirectColourDrawVariety) -> anyhow::Result<DirectColourDraw> {
        Ok(DirectColourDraw(variety.clone()))
    }

    pub(crate) fn block_colour(&self, campaign: &mut AccumulatorCampaign, colours: &[DirectColour], vertexes: usize) -> anyhow::Result<()> {
        let mut codes = vec![];
        for c in colours {
            for _ in 0..vertexes {
                codes.push(scale_colour(c.0));
                codes.push(scale_colour(c.1));
                codes.push(scale_colour(c.2));
            }
        }
        campaign.add_n(&self.0.colour,codes)?;
        Ok(())
    }
}
