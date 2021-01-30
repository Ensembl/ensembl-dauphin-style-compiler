use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProcessName;
use super::super::layers::patina::PatinaProcessName;
use crate::webgl::{ AttribHandle, ProtoProcess, AccumulatorCampaign, Program };
use peregrine_core::{ ShipEnd, ScreenEdge };
use super::super::layers::arrayutil::{ 
    interleave_rect_x, interleave_line_x, calculate_vertex, calculate_vertex_delta, sea_sign, quads,
    calculate_stretch_vertex_delta, calculate_stretch_vertex
};

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
        let len = base_x.len();
        let mut campaign = layer.make_campaign(&GeometryProcessName::Tape,&self.patina,len,&[0,3,1,2,1,3])?;
        let x1 = calculate_vertex_delta(len,&ship_x,&size_x,false);
        let x2 = calculate_vertex_delta(len,&ship_x,&size_x,true);
        let y1 = calculate_vertex(&sea_y,&ship_y,&size_y,false);
        let y2 = calculate_vertex(&sea_y,&ship_y,&size_y,true);
        let signs = vec![sea_sign(&sea_y);len*4];
        let vertexes = interleave_rect_x(&x1,&y1,&x2,&y2);
        let origins = quads(&base_x);
        campaign.add(&self.variety.origins,origins)?; /* 4n */
        campaign.add(&self.variety.vertexes,vertexes)?; /* 8n */
        campaign.add(&self.variety.signs,signs)?; /* 4n */
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
        let x1 = calculate_stretch_vertex_delta(len,&pxx1);
        let x2 = calculate_stretch_vertex_delta(len,&pxx2);
        let y1 = calculate_stretch_vertex(&ayy1,&pyy1);
        let y2 = calculate_stretch_vertex(&ayy2,&pyy2);
        let vertexes = interleave_rect_x(&x1,&y1,&x2,&y2);
        let origins = interleave_line_x(&axx1,&axx2);
        let signs = vec![1.;len*4];
        campaign.add(&self.variety.signs,signs)?; /* 4n */
        campaign.add(&self.variety.vertexes,vertexes)?; /* 8n */
        campaign.add(&self.variety.origins,origins)?; /* 4n */
        Ok(campaign)
    }
}
