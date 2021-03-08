use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProcessName;
use super::super::layers::patina::PatinaProcessName;
use crate::webgl::{ AttribHandle, ProtoProcess, ProcessStanzaElements, Program, ProcessStanzaAddable };
use peregrine_core::{ ShipEnd, ScreenEdge };
use super::super::util::glaxis::GLAxis;

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

    fn add(&self, layer: &mut Layer, x_axis: GLAxis, y_axis: GLAxis, x_origin: GLAxis) -> anyhow::Result<ProcessStanzaElements> {
        let mut elements = x_axis.make_elements(layer,&GeometryProcessName::Tape,&self.patina)?;
        elements.add(&self.variety.origins,x_origin.vec1d_x())?;
        elements.add(&self.variety.vertexes,x_axis.vec2d(&y_axis))?;
        elements.add(&self.variety.signs,y_axis.signs_y())?;
        Ok(elements)
    }

    pub(crate) fn add_rectangles(&self, layer: &mut Layer,
                                    base_x: Vec<f64>, sea_y: ScreenEdge,
                                    ship_x: ShipEnd, ship_y: ShipEnd,
                                    size_x: Vec<f64>, size_y: Vec<f64>, hollow: bool) -> anyhow::Result<ProcessStanzaElements> {
        let x_origin = GLAxis::new_single_origin(&base_x, -layer.left(), true,hollow);
        let x_axis = GLAxis::new_from_single_delta(x_origin.len(),&ship_x,&size_x,true,hollow);
        let y_axis = GLAxis::new_from_single(&sea_y,&ship_y,&size_y,false,hollow);
        self.add(layer,x_axis,y_axis,x_origin)
    }

    pub(crate) fn add_stretchtangle(&self, layer: &mut Layer, 
                    axx1: Vec<f64>, ayy1: ScreenEdge, /* sea-end anchor1 (mins) */
                    axx2: Vec<f64>, ayy2: ScreenEdge, /* sea-end anchor2 (maxes) */
                    pxx1: ShipEnd, pyy1: ShipEnd,       /* ship-end anchor1 */
                    pxx2: ShipEnd, pyy2: ShipEnd,       /* ship-end anchor2 */
                    hollow: bool) -> anyhow::Result<ProcessStanzaElements> {
        let x_origin = GLAxis::new_double_origin(&axx1,&axx2, -layer.left(), true,hollow);
        let x_axis = GLAxis::new_from_double_delta(x_origin.len(), &pxx1,&pxx2,true,hollow);
        let y_axis = GLAxis::new_from_double(&ayy1, &pyy1, &ayy2, &pyy2, false,hollow);
        self.add(layer,x_axis,y_axis,x_origin)
    }
}
