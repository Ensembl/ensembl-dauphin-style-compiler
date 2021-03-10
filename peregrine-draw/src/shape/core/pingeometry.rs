use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProcessName;
use super::super::layers::patina::PatinaProcessName;
use crate::webgl::{ AttribHandle, ProtoProcess, ProcessStanzaElements, Program, ProcessStanzaAddable };
use peregrine_data::{ ShipEnd };
use super::super::util::glaxis::GLAxis;
use crate::shape::core::stage::{ ReadStage };
use super::geometrydata::GeometryData;

pub struct PinData {
    x_vertex: GLAxis,
    y_vertex: GLAxis,
    x_origin: GLAxis,
    y_origin: GLAxis
}

impl PinData {
    pub(crate) fn add_rectangles(layer: &Layer, 
                                    base_x: Vec<f64>, base_y: Vec<f64>,
                                    ship_x: ShipEnd, ship_y: ShipEnd,
                                    size_x: Vec<f64>, size_y: Vec<f64>, hollow: bool) -> PinData {
        let x_origin = GLAxis::new_single_origin(&base_x,-layer.left(), true, hollow);
        PinData {
            x_vertex: GLAxis::new_from_single_delta(x_origin.len(),&ship_x,&size_x,true,hollow),
            y_vertex: GLAxis::new_from_single_delta(x_origin.len(),&ship_y,&size_y,false,hollow),
            x_origin,
            y_origin: GLAxis::new_single_origin(&base_y,0., false, hollow),
        }
    }

    pub(crate) fn add_stretchtangle(layer: &Layer, 
                                    axx1: Vec<f64>, ayy1: Vec<f64>, /* sea-end anchor1 (mins) */
                                    axx2: Vec<f64>, ayy2: Vec<f64>, /* sea-end anchor2 (maxes) */
                                    pxx1: ShipEnd, pyy1: ShipEnd,       /* ship-end anchor1 */
                                    pxx2: ShipEnd, pyy2: ShipEnd,       /* ship-end anchor2 */
                                    hollow: bool) -> PinData {
        let x_origin = GLAxis::new_double_origin(&axx1, &axx2,-layer.left(), true,hollow);
        PinData {
            x_vertex: GLAxis::new_from_double_delta(x_origin.len(), &pxx1, &pxx2, true,hollow),
            y_vertex: GLAxis::new_from_double_delta(x_origin.len(), &pyy1, &pyy2, false,hollow),
            x_origin,
            y_origin: GLAxis::new_double_origin(&ayy1, &ayy2,0., false,hollow),
        }
    }
}

impl GeometryData for PinData {
    fn iter_screen<'x>(&'x self, stage: &ReadStage) -> anyhow::Result<Box<dyn Iterator<Item=((f64,f64),(f64,f64))> + 'x>> {
        let x_vertex = self.x_vertex.iter_screen(stage.x())?;
        let x_origin = self.x_origin.iter_paper(stage.x())?;
        let x = x_vertex.zip(x_origin).map(|(s,p)| (s.0+p.0,s.1+p.1));
        let y_vertex = self.y_vertex.iter_screen(stage.y())?;
        let y_origin = self.y_origin.iter_paper(stage.y())?;
        let y = y_vertex.zip(y_origin).map(|(s,p)| (s.0+p.0,s.1+p.1));
        Ok(Box::new(x.zip(y)))
    }

    fn in_bounds(&self, stage: &ReadStage, mouse: (u32,u32)) -> anyhow::Result<bool> {
        let mouse = (mouse.0 as f64, mouse.1 as f64);
        let min_x = self.x_vertex.min_screen(stage.x())? - self.x_origin.min_paper(stage.x())?;
        let max_x = self.x_vertex.max_screen(stage.x())? + self.x_origin.max_paper(stage.x())?;
        let min_y = self.y_vertex.min_screen(stage.y())? - self.y_origin.min_paper(stage.y())?;
        let max_y = self.y_vertex.max_screen(stage.y())? + self.y_origin.max_paper(stage.y())?;
        Ok(!(mouse.0 < min_x || mouse.0 > max_x || mouse.1 < min_y || mouse.1 > max_y))
    }
}

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

    pub(crate) fn add(&self, layer: &mut Layer, data: PinData) -> anyhow::Result<ProcessStanzaElements> {
        let mut elements = data.x_origin.make_elements(layer, &GeometryProcessName::Pin,&self.patina)?;
        elements.add(&self.variety.origins,data.x_origin.vec2d(&data.y_origin))?;
        elements.add(&self.variety.vertexes,data.x_vertex.vec2d(&data.y_vertex))?;
        Ok(elements)
    }
}
