use crate::webgl::{ AttribHandle, ProcessBuilder, AccumulatorCampaign };
use peregrine_core::DirectColour;

#[derive(Clone)]
pub struct DirectColourDraw {
    colour: AttribHandle
}

fn scale_colour(value: u8) -> f64 {
    (value as f64)/255.
}

impl DirectColourDraw {
    pub(crate) fn new(process: &ProcessBuilder) -> anyhow::Result<DirectColourDraw> {
        let colour = process.get_attrib_handle("aVertexColour")?;
        Ok(DirectColourDraw { colour })
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
        campaign.add_n(&self.colour,codes)?;
        Ok(())
    }
}
