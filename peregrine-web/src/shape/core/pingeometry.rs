use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProcessName;
use super::super::layers::patina::PatinaProcessName;
use super::arrayutil::{ interleave_rect_x, make_rect_elements, calculate_vertex_delta, calculate_stretch_vertex_delta, apply_left };
use crate::webgl::{ AttribHandle, ProtoProcess, ProcessStanzaElements, Program, ProcessStanzaAddable };
use peregrine_core::{ ShipEnd };

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

const HOLLOW_WIDTH : f64 = 1.; // XXX

impl PinGeometry {
    pub(crate) fn new(_process: &ProtoProcess, patina: &PatinaProcessName, variety: &PinProgram) -> anyhow::Result<PinGeometry> {
        Ok(PinGeometry { variety: variety.clone(), patina: patina.clone() })
    }

    pub(crate) fn add_rectangles(&self, layer: &mut Layer,
                                    mut base_x: Vec<f64>, base_y: Vec<f64>,
                                    ship_x: ShipEnd, ship_y: ShipEnd,
                                    size_x: Vec<f64>, size_y: Vec<f64>, hollow: bool) -> anyhow::Result<ProcessStanzaElements> {
        let len = base_x.len();
        let mut elements = make_rect_elements(layer,&GeometryProcessName::Tape,&self.patina,len,hollow)?;
        let x1 = calculate_vertex_delta(len,&ship_x,&size_x,false);
        let x2 = calculate_vertex_delta(len,&ship_x,&size_x,true); 
        let y1 = calculate_vertex_delta(len,&ship_y,&size_y,false);
        let y2 = calculate_vertex_delta(len,&ship_y,&size_y,true);
        let vertexes = interleave_rect_x(&x1,&y1,&x2,&y2,if hollow {Some(HOLLOW_WIDTH)} else {None});
        apply_left(&mut base_x,layer);
        let origins = interleave_rect_x(&base_x,&base_y,&base_x,&base_y,if hollow {Some(0.)} else {None});
        elements.add(&self.variety.origins,origins)?; /* 8n */
        elements.add(&self.variety.vertexes,vertexes)?; /* 8n */
        Ok(elements)
    }

    pub(crate) fn add_stretchtangle(&self, layer: &mut Layer, 
                mut axx1: Vec<f64>, ayy1: Vec<f64>, /* sea-end anchor1 (mins) */
                mut axx2: Vec<f64>, ayy2: Vec<f64>, /* sea-end anchor2 (maxes) */
                pxx1: ShipEnd, pyy1: ShipEnd,       /* ship-end anchor1 */
                pxx2: ShipEnd, pyy2: ShipEnd,       /* ship-end anchor2 */
                hollow: bool) -> anyhow::Result<ProcessStanzaElements> {
        let len = axx1.len();
        let mut elements = make_rect_elements(layer,&GeometryProcessName::Tape,&self.patina,len,hollow)?;
        let x1 = calculate_stretch_vertex_delta(len,&pxx1);
        let y1 = calculate_stretch_vertex_delta(len,&pyy1);
        let x2 = calculate_stretch_vertex_delta(len,&pxx2);
        let y2 = calculate_stretch_vertex_delta(len,&pyy2);
        let vertexes = interleave_rect_x(&x1,&y1,&x2,&y2,if hollow {Some(HOLLOW_WIDTH)} else {None});
        apply_left(&mut axx1,layer);
        apply_left(&mut axx2,layer);
        let origins = interleave_rect_x(&axx1,&ayy1,&axx2,&ayy2,if hollow {Some(0.)} else {None});
        elements.add(&self.variety.vertexes,vertexes)?; /* 8n */
        elements.add(&self.variety.origins,origins)?; /* 8n */
        Ok(elements)
    }
}
