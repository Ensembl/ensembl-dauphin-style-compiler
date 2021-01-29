use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProcessName;
use super::super::layers::patina::PatinaProcessName;
use super::super::layers::arrayutil::{ add_fixed_sea_box, ship_box, interleave_one };
use crate::webgl::{ AttribHandle, ProtoProcess, AccumulatorCampaign, Program };
use peregrine_core::{ ShipEnd, ScreenEdge };
use super::super::layers::arrayutil::{ stretchtangle, repeat, interleave_rect_x };

#[derive(Clone)]
pub struct FixProgram {
    vertexes: AttribHandle,
    signs: AttribHandle,
}

impl FixProgram {
    pub(crate) fn new(program: &Program) -> anyhow::Result<FixProgram> {
        Ok(FixProgram {
            vertexes: program.get_attrib_handle("aVertexPosition")?,
            signs: program.get_attrib_handle("aSign")?
        })
    }
}

#[derive(Clone)]
pub struct FixGeometry {
    variety: FixProgram,
    patina: PatinaProcessName
}

impl FixGeometry {
    pub(crate) fn new(_process: &ProtoProcess, patina: &PatinaProcessName, variety: &FixProgram) -> anyhow::Result<FixGeometry> {
        Ok(FixGeometry { variety: variety.clone(), patina: patina.clone() })
    }

    pub(crate) fn add_solid_rectangles(&self, layer: &mut Layer,
                                        sea_x: ScreenEdge, sea_y: ScreenEdge,
                                        ship_x: ShipEnd, ship_y: ShipEnd,
                                        size_x: Vec<f64>, size_y: Vec<f64>) -> anyhow::Result<AccumulatorCampaign> {
        let len = sea_x.len();
        let mut campaign = layer.make_campaign(&GeometryProcessName::Fix,&self.patina,len,&[0,3,1,2,1,3])?;
        let sign_x = match sea_x { ScreenEdge::Max(_) => -1., _ => 1. };
        let sign_y = match sea_y { ScreenEdge::Max(_) => -1., _ => 1. };
        let mut vertexes = ship_box(ship_x,size_x,ship_y,size_y,len);
        add_fixed_sea_box(&mut vertexes,false,sea_x);
        add_fixed_sea_box(&mut vertexes,true,sea_y);
        campaign.add(&self.variety.vertexes,vertexes)?;
        campaign.add(&self.variety.signs,interleave_one(sign_x,sign_y,len)?)?;
        Ok(campaign)
    }

    pub(crate) fn add_solid_stretchtangle(&self, layer: &mut Layer, 
                                            axx1: ScreenEdge, ayy1: ScreenEdge, /* sea-end anchor1 (mins) */
                                            axx2: ScreenEdge, ayy2: ScreenEdge, /* sea-end anchor2 (maxes) */
                                            pxx1: ShipEnd, pyy1: ShipEnd,       /* ship-end anchor1 */
                                            pxx2: ShipEnd, pyy2: ShipEnd,       /* ship-end anchor2 */
                                        ) -> anyhow::Result<AccumulatorCampaign> {
        let len = axx1.len();
        let mut campaign = layer.make_campaign(&GeometryProcessName::Fix,&self.patina,len,&[0,3,1,2,1,3])?;
        let (axx1,sxx1) = stretchtangle(axx1,pxx1,false)?;
        let (ayy1,syy1) = stretchtangle(ayy1,pyy1,false)?;
        let (axx2,sxx2) = stretchtangle(axx2,pxx2,true)?;
        let (ayy2,syy2) = stretchtangle(ayy2,pyy2,true)?;
        let vertexes = interleave_rect_x(&axx1,&ayy1,&axx2,&ayy2);
        let signs = repeat(&[sxx1,syy1,  sxx1,syy2,   sxx2,syy1,   sxx2,syy2],len);
        campaign.add(&self.variety.vertexes,vertexes)?;
        campaign.add(&self.variety.signs,signs)?;
        Ok(campaign)
    }
}
