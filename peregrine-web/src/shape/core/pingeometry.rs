use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProcessName;
use super::super::layers::patina::PatinaProcessName;
use crate::webgl::{ AttribHandle, ProtoProcess, ProcessStanzaElements, Program, ProcessStanzaAddable };
use peregrine_core::{ ShipEnd };
use super::super::util::glaxis::GLAxis;


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

    fn add(&self, layer: &mut Layer, x_axis: GLAxis, y_axis: GLAxis, x_origin_axis: GLAxis, y_origin_axis: GLAxis) -> anyhow::Result<ProcessStanzaElements> {
        let mut elements = x_axis.make_elements(layer, &GeometryProcessName::Pin,&self.patina)?;
        elements.add(&self.variety.origins,x_origin_axis.vec2d(&y_origin_axis))?;
        elements.add(&self.variety.vertexes,x_axis.vec2d(&y_axis))?;
        Ok(elements)
    }

    pub(crate) fn add_rectangles(&self, layer: &mut Layer,
                                    base_x: Vec<f64>, base_y: Vec<f64>,
                                    ship_x: ShipEnd, ship_y: ShipEnd,
                                    size_x: Vec<f64>, size_y: Vec<f64>, hollow: bool) -> anyhow::Result<ProcessStanzaElements> {
        let x_origin_axis = GLAxis::new_single_origin(&base_x,-layer.left(), true, hollow);
        let y_origin_axis = GLAxis::new_single_origin(&base_y,0., false, hollow);
        let x_axis = GLAxis::new_from_single_delta(x_origin_axis.len(),&ship_x,&size_x,true,hollow);
        let y_axis = GLAxis::new_from_single_delta(x_origin_axis.len(),&ship_y,&size_y,false,hollow);
        self.add(layer,x_axis,y_axis,x_origin_axis,y_origin_axis)
    }

    pub(crate) fn add_stretchtangle(&self, layer: &mut Layer, 
                axx1: Vec<f64>, ayy1: Vec<f64>, /* sea-end anchor1 (mins) */
                axx2: Vec<f64>, ayy2: Vec<f64>, /* sea-end anchor2 (maxes) */
                pxx1: ShipEnd, pyy1: ShipEnd,       /* ship-end anchor1 */
                pxx2: ShipEnd, pyy2: ShipEnd,       /* ship-end anchor2 */
                hollow: bool) -> anyhow::Result<ProcessStanzaElements> {
        let x_origin_axis = GLAxis::new_double_origin(&axx1, &axx2,-layer.left(), true,hollow);
        let y_origin_axis = GLAxis::new_double_origin(&ayy1, &ayy2,0., false,hollow);
        let x_axis = GLAxis::new_from_double_delta(x_origin_axis.len(), &pxx1, &pxx2, true,hollow);
        let y_axis = GLAxis::new_from_double_delta(x_origin_axis.len(), &pyy1, &pyy2, false,hollow);
        self.add(layer,x_axis,y_axis,x_origin_axis,y_origin_axis)
    }
}
