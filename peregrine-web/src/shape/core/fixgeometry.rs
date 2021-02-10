use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProcessName;
use super::super::layers::patina::PatinaProcessName;
use crate::webgl::{ AttribHandle, ProtoProcess, ProcessStanzaElements, Program, ProcessStanzaAddable };
use peregrine_core::{ ShipEnd, ScreenEdge };
use super::arrayutil::{ repeat, interleave_rect_x, calculate_vertex, sea_sign, calculate_stretch_vertex, interleave_pair_count };

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
                                        size_x: Vec<f64>, size_y: Vec<f64>) -> anyhow::Result<ProcessStanzaElements> {
        let len = sea_x.len();
        let mut elements = layer.make_elements(&GeometryProcessName::Fix,&self.patina,len,&[0,3,1,2,1,3])?;
        let x1 = calculate_vertex(&sea_x,&ship_x,&size_x,false);
        let x2 = calculate_vertex(&sea_x,&ship_x,&size_x,true);
        let y1 = calculate_vertex(&sea_y,&ship_y,&size_y,false);
        let y2 = calculate_vertex(&sea_y,&ship_y,&size_y,true);
        let signs = interleave_pair_count(sea_sign(&sea_x),sea_sign(&sea_y),len*4)?;
        let vertexes = interleave_rect_x(&x1,&y1,&x2,&y2);
        elements.add(&self.variety.vertexes,vertexes)?; /* 8n */
        elements.add(&self.variety.signs,signs)?; /* 8n */
        Ok(elements)
    }

    pub(crate) fn add_solid_stretchtangle(&self, layer: &mut Layer, 
                                            axx1: ScreenEdge, ayy1: ScreenEdge, /* sea-end anchor1 (mins) */
                                            axx2: ScreenEdge, ayy2: ScreenEdge, /* sea-end anchor2 (maxes) */
                                            pxx1: ShipEnd, pyy1: ShipEnd,       /* ship-end anchor1 */
                                            pxx2: ShipEnd, pyy2: ShipEnd,       /* ship-end anchor2 */
                                        ) -> anyhow::Result<ProcessStanzaElements> {
        let len = axx1.len();
        let mut elements = layer.make_elements(&GeometryProcessName::Fix,&self.patina,len,&[0,3,1,2,1,3])?;
        let x1 = calculate_stretch_vertex(&axx1,&pxx1);
        let x2 = calculate_stretch_vertex(&axx2,&pxx2);
        let y1 = calculate_stretch_vertex(&ayy1,&pyy1);
        let y2 = calculate_stretch_vertex(&ayy2,&pyy2);
        let vertexes = interleave_rect_x(&x1,&y1,&x2,&y2);
        let sx1 = sea_sign(&axx1);
        let sx2 = sea_sign(&axx2);
        let sy1 = sea_sign(&ayy1);
        let sy2 = sea_sign(&ayy2);
        let signs = repeat(&[sx1,sy1,  sx1,sy2,   sx2,sy1,   sx2,sy2],len);
        elements.add(&self.variety.vertexes,vertexes)?; /* 8n */
        elements.add(&self.variety.signs,signs)?; /* 8n */
        Ok(elements)
    }
}
