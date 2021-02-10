use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProcessName;
use super::super::layers::patina::PatinaProcessName;
use super::arrayutil::{ interleave_pair_count };
use crate::webgl::{ AttribHandle, ProtoProcess, AccumulatorCampaign, Program, AccumulatorAddable };
use peregrine_core::{ ShipEnd, ScreenEdge };
use super::arrayutil::{ repeat, interleave_rect_y, calculate_vertex, sea_sign, calculate_vertex_min, calculate_stretch_vertex };

#[derive(Clone)]
pub struct PageProgram {
    vertexes: AttribHandle,
    signs: AttribHandle,
}

impl PageProgram {
    pub(crate) fn new(program: &Program) -> anyhow::Result<PageProgram> {
        Ok(PageProgram {
            vertexes: program.get_attrib_handle("aVertexPosition")?,
            signs: program.get_attrib_handle("aSign")?
        })
    }
}

#[derive(Clone)]
pub struct PageGeometry {
    variety: PageProgram,
    patina: PatinaProcessName
}

impl PageGeometry {
    pub(crate) fn new(_process: &ProtoProcess, patina: &PatinaProcessName, variety: &PageProgram) -> anyhow::Result<PageGeometry> {
        Ok(PageGeometry { variety: variety.clone(), patina: patina.clone() })
    }

    pub(crate) fn add_solid_rectangles(&self, layer: &mut Layer,
                                        sea_x: ScreenEdge, yy: Vec<f64>,
                                        ship_x: ShipEnd, ship_y: ShipEnd,
                                        size_x: Vec<f64>, size_y: Vec<f64>) -> anyhow::Result<AccumulatorCampaign> {
        let len = yy.len();
        let mut campaign = layer.make_campaign(&GeometryProcessName::Page,&self.patina,len,&[0,3,1,2,1,3])?;
        let x1 = calculate_vertex(&sea_x,&ship_x,&size_x,false);
        let x2 = calculate_vertex(&sea_x,&ship_x,&size_x,true);
        let y1 = calculate_vertex_min(&yy,&ship_y,&size_y,false);
        let y2 = calculate_vertex_min(&yy,&ship_y,&size_y,true);
        let signs = interleave_pair_count(sea_sign(&sea_x),1.,len*4)?;
        let vertexes = interleave_rect_y(&x1,&y1,&x2,&y2);
        campaign.add(&self.variety.vertexes,vertexes)?; /* 8n */
        campaign.add(&self.variety.signs,signs)?; /* 8n */
        Ok(campaign)
    }

    pub(crate) fn add_solid_stretchtangle(&self, layer: &mut Layer, 
        axx1: ScreenEdge, ayy1: Vec<f64>, /* sea-end anchor1 (mins) */
        axx2: ScreenEdge, ayy2: Vec<f64>, /* sea-end anchor2 (maxes) */
        pxx1: ShipEnd, pyy1: ShipEnd,      /* ship-end anchor1 */
        pxx2: ShipEnd, pyy2: ShipEnd,      /* ship-end anchor2 */
                ) -> anyhow::Result<AccumulatorCampaign> {
        let len = ayy1.len();
        let mut campaign = layer.make_campaign(&GeometryProcessName::Page,&self.patina,len,&[0,3,1,2,1,3])?;
        let x1 = calculate_stretch_vertex(&axx1,&pxx1);
        let x2 = calculate_stretch_vertex(&axx2,&pxx2);
        let y1 = calculate_stretch_vertex(&ScreenEdge::Min(ayy1),&pyy1);
        let y2 = calculate_stretch_vertex(&ScreenEdge::Min(ayy2),&pyy2);
        let vertexes = interleave_rect_y(&x1,&y1,&x2,&y2);
        let sx1 = sea_sign(&axx1);
        let sx2 = sea_sign(&axx2);
        let signs = repeat(&[sx1,1., sx1,1., sx2,1., sx2,1.],len);
        campaign.add(&self.variety.vertexes,vertexes)?; /* 8n */
        campaign.add(&self.variety.signs,signs)?; /* 8n */
        Ok(campaign)
    }
}
