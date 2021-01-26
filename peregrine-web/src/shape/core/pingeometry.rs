use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProcessName;
use super::super::layers::patina::PatinaProcessName;
use super::super::layers::arrayutil::{ interleave, ship_box };
use crate::webgl::{ AttribHandle, ProtoProcess, AccumulatorCampaign, Program };
use peregrine_core::ShipEnd;

#[derive(Clone)]
pub struct PinProgram {
    vertexes: AttribHandle,
    origins: AttribHandle,
}

impl PinProgram {
    pub(crate) fn new(program: &Program) -> anyhow::Result<PinProgram> {
        Ok(PinProgram {
            vertexes: program.get_attrib_handle("aVertexPosition")?,
            origins: program.get_attrib_handle("aOrigin")?
        })
    }
}

#[derive(Clone)]
pub struct PinGeometry {
    variety: PinProgram,
    patina: PatinaProcessName
}

impl PinGeometry {
    pub(crate) fn new(_process: &ProtoProcess, patina: &PatinaProcessName, variety: &PinProgram) -> anyhow::Result<PinGeometry> {
        Ok(PinGeometry { variety: variety.clone(), patina: patina.clone() })
    }

    pub(crate) fn add_solid_rectangles(&self, layer: &mut Layer,
                                        base_x: Vec<f64>, base_y: Vec<f64>,
                                        ship_x: ShipEnd, ship_y: ShipEnd,
                                        size_x: Vec<f64>, size_y: Vec<f64>) -> anyhow::Result<AccumulatorCampaign> {
        let mut campaign = layer.make_campaign(&GeometryProcessName::Pin,&self.patina,base_x.len(),&[0,3,1,2,1,3])?;
        let len = base_x.len();
        campaign.add(&self.variety.origins,interleave(base_x,&base_y)?)?;
        campaign.add(&self.variety.vertexes,ship_box(ship_x,size_x,ship_y,size_y,len))?;
        Ok(campaign)
    }
}
