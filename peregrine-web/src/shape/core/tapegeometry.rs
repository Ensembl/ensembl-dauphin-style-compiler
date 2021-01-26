use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryAccessorName;
use super::super::layers::patina::PatinaAccessorName;
use super::super::layers::arrayutil::{ add_fixed_sea_box, ship_box };
use crate::webgl::{ AttribHandle, ProtoProcess, AccumulatorCampaign, Program };
use peregrine_core::{ ShipEnd, ScreenEdge };

#[derive(Clone)]
pub struct TapeGeometryVariety {
    origin: AttribHandle,
    vertexes: AttribHandle,
    signs: AttribHandle,
}

impl TapeGeometryVariety {
    pub(crate) fn new(program: &Program) -> anyhow::Result<TapeGeometryVariety> {
        Ok(TapeGeometryVariety {
            origin: program.get_attrib_handle("aOrigin")?,
            vertexes: program.get_attrib_handle("aVertexPosition")?,
            signs: program.get_attrib_handle("aSign")?
        })
    }
}

#[derive(Clone)]
pub struct TapeGeometry {
    variety: TapeGeometryVariety,
    patina: PatinaAccessorName
}

impl TapeGeometry {
    pub(crate) fn new(process: &ProtoProcess, patina: &PatinaAccessorName, variety: &TapeGeometryVariety) -> anyhow::Result<TapeGeometry> {
        Ok(TapeGeometry { variety: variety.clone(), patina: patina.clone() })
    }

    pub(crate) fn add_solid_rectangles(&self, layer: &mut Layer,
                                        base_x: Vec<f64>, sea_y: ScreenEdge,
                                        ship_x: ShipEnd, ship_y: ShipEnd,
                                        size_x: Vec<f64>, size_y: Vec<f64>) -> anyhow::Result<AccumulatorCampaign> {
        let mut campaign = layer.make_campaign(&GeometryAccessorName::Tape,&self.patina,base_x.len(),&[0,3,1,2,1,3])?;
        let len = base_x.len();
        let mut vertexes = ship_box(ship_x,size_x,ship_y,size_y,len);
        let sign_y = match sea_y { ScreenEdge::Max(_) => -1., _ => 1. };
        let y = match &sea_y { ScreenEdge::Min(z) => z, ScreenEdge::Max(z) => z };
        add_fixed_sea_box(&mut vertexes,true,sea_y);
        campaign.add(&self.variety.origin,base_x)?;
        campaign.add(&self.variety.vertexes,vertexes)?;
        campaign.add(&self.variety.signs,vec![sign_y;len])?;        
        Ok(campaign)
    }
}
