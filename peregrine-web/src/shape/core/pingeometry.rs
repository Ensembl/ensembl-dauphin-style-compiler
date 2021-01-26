use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryAccessorName;
use super::super::layers::patina::PatinaAccessorName;
use super::super::layers::arrayutil::{ interleave, ship_box };
use crate::webgl::{ AttribHandle, ProcessBuilder, AccumulatorCampaign };
use peregrine_core::ShipEnd;

#[derive(Clone)]
pub struct PinGeometry {
    vertexes: AttribHandle,
    origins: AttribHandle,
    patina: PatinaAccessorName
}

impl PinGeometry {
    pub(crate) fn new(process: &ProcessBuilder, patina: &PatinaAccessorName) -> anyhow::Result<PinGeometry> {
        let vertexes = process.get_attrib_handle("aVertexPosition")?;
        let origins = process.get_attrib_handle("aOrigin")?;
        Ok(PinGeometry { vertexes, origins, patina: patina.clone() })
    }

    pub(crate) fn add_solid_rectangles(&self, layer: &mut Layer,
                                        base_x: Vec<f64>, base_y: Vec<f64>,
                                        ship_x: ShipEnd, ship_y: ShipEnd,
                                        size_x: Vec<f64>, size_y: Vec<f64>) -> anyhow::Result<AccumulatorCampaign> {
        let mut campaign = layer.make_campaign(&GeometryAccessorName::Pin,&self.patina,base_x.len(),&[0,3,1,2,1,3])?;
        let len = base_x.len();
        campaign.add(&self.origins,interleave(base_x,&base_y)?)?;
        campaign.add(&self.vertexes,ship_box(ship_x,size_x,ship_y,size_y,len))?;
        Ok(campaign)
    }
}
