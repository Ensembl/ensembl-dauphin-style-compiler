use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryAccessorName;
use super::super::layers::patina::PatinaAccessorName;
use super::super::layers::arrayutil::{ add_fixed_sea_box, ship_box };
use crate::webgl::{ AttribHandle, ProcessBuilder, AccumulatorCampaign };
use peregrine_core::{ ShipEnd, ScreenEdge };

#[derive(Clone)]
pub struct TapeGeometry {
    origin: AttribHandle,
    vertexes: AttribHandle,
    signs: AttribHandle,
    patina: PatinaAccessorName
}

impl TapeGeometry {
    pub(crate) fn new(process: &ProcessBuilder, patina: &PatinaAccessorName) -> anyhow::Result<TapeGeometry> {
        let origin = process.get_attrib_handle("aOrigin")?;
        let vertexes = process.get_attrib_handle("aVertexPosition")?;
        let signs = process.get_attrib_handle("aSign")?;
        Ok(TapeGeometry { vertexes, signs, origin, patina: patina.clone() })
    }

    pub(crate) fn add_solid_rectangles(&self, layer: &mut Layer,
                                        base_x: Vec<f64>, sea_y: ScreenEdge,
                                        ship_x: ShipEnd, ship_y: ShipEnd,
                                        size_x: Vec<f64>, size_y: Vec<f64>) -> anyhow::Result<AccumulatorCampaign> {
        let mut campaign = layer.make_campaign(&GeometryAccessorName::Tape,&self.patina,base_x.len(),&[0,3,1,2,1,3])?;
        let len = base_x.len();
        campaign.add(&self.origin,base_x)?;
        let mut vertexes = ship_box(ship_x,size_x,ship_y,size_y,len);
        let sign_y = match sea_y { ScreenEdge::Max(_) => -1., _ => 1. };
        let y = match &sea_y { ScreenEdge::Min(z) => z, ScreenEdge::Max(z) => z };
        add_fixed_sea_box(&mut vertexes,true,sea_y);
        campaign.add(&self.vertexes,vertexes)?;
        campaign.add(&self.signs,vec![sign_y;len])?;        
        Ok(campaign)
    }
}
