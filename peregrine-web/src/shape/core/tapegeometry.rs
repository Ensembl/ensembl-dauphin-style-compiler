use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProcessName;
use super::super::layers::patina::PatinaProcessName;
use super::super::layers::arrayutil::{ add_fixed_sea_box, ship_box };
use crate::webgl::{ AttribHandle, ProtoProcess, AccumulatorCampaign, Program };
use peregrine_core::{ ShipEnd, ScreenEdge };
use super::super::layers::arrayutil::{ stretchtangle, interleave_rect_x, interleave_line_x };

#[derive(Clone)]
pub struct TapeProgram {
    origins: AttribHandle,
    vertexes: AttribHandle,
    signs: AttribHandle,
}

impl TapeProgram {
    pub(crate) fn new(program: &Program) -> anyhow::Result<TapeProgram> {
        Ok(TapeProgram {
            origins: program.get_attrib_handle("aOrigin")?,
            vertexes: program.get_attrib_handle("aVertexPosition")?,
            signs: program.get_attrib_handle("aSign")?
        })
    }
}

#[derive(Clone)]
pub struct TapeGeometry {
    variety: TapeProgram,
    patina: PatinaProcessName
}

impl TapeGeometry {
    pub(crate) fn new(_process: &ProtoProcess, patina: &PatinaProcessName, variety: &TapeProgram) -> anyhow::Result<TapeGeometry> {
        Ok(TapeGeometry { variety: variety.clone(), patina: patina.clone() })
    }

    pub(crate) fn add_solid_rectangles(&self, layer: &mut Layer,
                                        base_x: Vec<f64>, sea_y: ScreenEdge,
                                        ship_x: ShipEnd, ship_y: ShipEnd,
                                        size_x: Vec<f64>, size_y: Vec<f64>) -> anyhow::Result<AccumulatorCampaign> {
        let mut campaign = layer.make_campaign(&GeometryProcessName::Tape,&self.patina,base_x.len(),&[0,3,1,2,1,3])?;
        let len = base_x.len();
        let mut vertexes = ship_box(ship_x,size_x,ship_y,size_y,len);
        let sign_y = match sea_y { ScreenEdge::Max(_) => -1., _ => 1. };
        let y = match &sea_y { ScreenEdge::Min(z) => z, ScreenEdge::Max(z) => z };
        add_fixed_sea_box(&mut vertexes,true,sea_y);
        campaign.add(&self.variety.origins,base_x)?;
        campaign.add(&self.variety.vertexes,vertexes)?;
        campaign.add(&self.variety.signs,vec![sign_y;len])?;        
        Ok(campaign)
    }

    pub(crate) fn add_solid_stretchtangle(&self, layer: &mut Layer, 
            axx1: Vec<f64>, ayy1: ScreenEdge, /* sea-end anchor1 (mins) */
            axx2: Vec<f64>, ayy2: ScreenEdge, /* sea-end anchor2 (maxes) */
            pxx1: ShipEnd, pyy1: ShipEnd,       /* ship-end anchor1 */
            pxx2: ShipEnd, pyy2: ShipEnd,       /* ship-end anchor2 */
                    ) -> anyhow::Result<AccumulatorCampaign> {
        let len = axx1.len();
        let mut campaign = layer.make_campaign(&GeometryProcessName::Tape,&self.patina,len,&[0,3,1,2,1,3])?;
        let (ayy1,_syy1) = stretchtangle(ayy1,pyy1,false)?;
        let (ayy2,_syy2) = stretchtangle(ayy2,pyy2,true)?;
        let pxx1 = match pxx1 { ShipEnd::Min(x) => x, ShipEnd::Centre(x) => x, ShipEnd::Max(x) => x };
        let pxx2 = match pxx2 { ShipEnd::Min(x) => x, ShipEnd::Centre(x) => x, ShipEnd::Max(x) => x };
        let vertexes = interleave_rect_x(&pxx1,&ayy1,&pxx2,&ayy2);
        let signs = vec![1.;len];
        let origins = interleave_line_x(&axx2,&axx2);
        campaign.add(&self.variety.signs,signs)?;
        campaign.add(&self.variety.vertexes,vertexes)?;
        campaign.add(&self.variety.origins,origins)?;
        Ok(campaign)
    }
}
