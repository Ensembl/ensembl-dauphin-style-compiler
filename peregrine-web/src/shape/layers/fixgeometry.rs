use super::super::core::paintgeometry::PaintGeometry;
use super::super::core::paintskin::PaintSkin;
use super::layer::Layer;
use super::arrayutil::{ add_fixed_sea_box, ship_box, interleave_one };
use crate::webgl::{ AttribHandle, ProcessBuilder, AccumulatorCampaign };
use peregrine_core::{ ShipEnd, ScreenEdge };

#[derive(Clone)]
pub struct FixGeometry {
    vertexes: AttribHandle,
    signs: AttribHandle
}

impl FixGeometry {
    pub(crate) fn new(process: &ProcessBuilder) -> anyhow::Result<FixGeometry> {
        let vertexes = process.get_attrib_handle("aVertexPosition")?;
        let signs = process.get_attrib_handle("aSign")?;
        Ok(FixGeometry { vertexes, signs })
    }

    pub(crate) fn add_solid_rectangles(&self, layer: &mut Layer, skin: &PaintSkin,
                                        sea_x: ScreenEdge, sea_y: ScreenEdge,
                                        ship_x: ShipEnd, ship_y: ShipEnd,
                                        size_x: Vec<f64>, size_y: Vec<f64>) -> anyhow::Result<AccumulatorCampaign> {
        let mut campaign = layer.make_campaign(&PaintGeometry::Fix,skin,sea_x.len(),&[0,3,1,2,1,3])?;
        let len = sea_x.len();
        let sign_x = match sea_x { ScreenEdge::Max(_) => -1., _ => 1. };
        let sign_y = match sea_y { ScreenEdge::Max(_) => -1., _ => 1. };
        let mut vertexes = ship_box(ship_x,size_x,ship_y,size_y,len);
        add_fixed_sea_box(&mut vertexes,false,sea_x);
        add_fixed_sea_box(&mut vertexes,true,sea_y);
        campaign.add(&self.vertexes,vertexes)?;
        campaign.add(&self.signs,interleave_one(sign_x,sign_y,len)?)?;
        Ok(campaign)
    }
}
