use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProcessName;
use super::super::layers::patina::PatinaProcessName;
use super::super::layers::arrayutil::{ interleave, ship_box, interleave_line_x, stretchtangle, interleave_rect_x };
use crate::webgl::{ AttribHandle, ProtoProcess, AccumulatorCampaign, Program };
use peregrine_core::{ ShipEnd, ScreenEdge };

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

    pub(crate) fn add_solid_stretchtangle(&self, layer: &mut Layer, 
                axx1: Vec<f64>, ayy1: Vec<f64>, /* sea-end anchor1 (mins) */
                axx2: Vec<f64>, ayy2: Vec<f64>, /* sea-end anchor2 (maxes) */
                pxx1: ShipEnd, pyy1: ShipEnd,       /* ship-end anchor1 */
                pxx2: ShipEnd, pyy2: ShipEnd,       /* ship-end anchor2 */
                        ) -> anyhow::Result<AccumulatorCampaign> {
            let len = axx1.len();
            let mut campaign = layer.make_campaign(&GeometryProcessName::Tape,&self.patina,len,&[0,3,1,2,1,3])?;
            let pxx1 = match pxx1 { ShipEnd::Min(x) => x, ShipEnd::Centre(x) => x, ShipEnd::Max(x) => x };
            let pxx2 = match pxx2 { ShipEnd::Min(x) => x, ShipEnd::Centre(x) => x, ShipEnd::Max(x) => x };
            let origins = interleave_line_x(&axx2,&axx2);
            let (ayy1,_syy1) = stretchtangle(ScreenEdge::Min(ayy1),pyy1,false)?;
            let (ayy2,_syy2) = stretchtangle(ScreenEdge::Min(ayy2),pyy2,true)?;
            let vertexes = interleave_rect_x(&pxx1,&ayy1,&pxx2,&ayy2);
            campaign.add(&self.variety.vertexes,vertexes)?;
            campaign.add(&self.variety.origins,origins)?;
            Ok(campaign)
        }
}
