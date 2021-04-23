use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProgramName;
use super::super::layers::patina::PatinaProcessName;
use crate::webgl::{ AttribHandle, ProtoProcess, ProcessStanzaElements, Program, ProcessStanzaAddable, GPUSpec, ProgramBuilder };
use peregrine_data::{ ShipEnd, ScreenEdge };
use super::super::util::glaxis::GLAxis;
use super::geometrydata::GeometryData;
use crate::shape::core::stage::{ ReadStage };
use crate::util::message::Message;
use web_sys::WebGlRenderingContext;

#[derive(Clone)]
pub struct TapeProgram {
    origins: AttribHandle,
    vertexes: AttribHandle,
    signs: AttribHandle,
}

impl TapeProgram {
    pub(crate) fn new(builder: &ProgramBuilder) -> Result<TapeProgram,Message> {
        Ok(TapeProgram {
            origins: builder.get_attrib_handle("aOrigin")?,
            vertexes: builder.get_attrib_handle("aVertexPosition")?,
            signs: builder.get_attrib_handle("aSign")?
        })
    }

    pub(crate) fn add(&self, process: &mut ProtoProcess, data: TapeData) -> Result<ProcessStanzaElements,Message> {
        let mut elements = data.x_origin.make_elements(process)?;
        elements.add(&self.origins,data.x_origin.vec1d_x(),1)?;
        elements.add(&self.vertexes,data.x_vertex.vec2d(&data.y_vertex),2)?;
        elements.add(&self.signs,data.y_vertex.signs_y(),1)?;
        Ok(elements)
    }
}

pub struct TapeData {
    x_vertex: GLAxis,
    y_vertex: GLAxis,
    x_origin: GLAxis
}

impl TapeData {
    pub(crate) fn add_rectangles(layer: &Layer,
                                    base_x: Vec<f64>, sea_y: ScreenEdge,
                                    ship_x: ShipEnd, ship_y: ShipEnd,
                                    size_x: Vec<f64>, size_y: Vec<f64>, hollow: bool) -> TapeData {
        let x_origin = GLAxis::new_single_origin(&base_x, -layer.left(), None,hollow);
        TapeData {
            x_vertex: GLAxis::new_from_single_delta(&ship_x,&size_x,&x_origin,hollow),
            y_vertex: GLAxis::new_from_single(&sea_y,&ship_y,&size_y,Some(&x_origin),hollow),
            x_origin,
        }
    }

    pub(crate) fn add_stretchtangle(layer: &Layer, 
                                        axx1: Vec<f64>, ayy1: ScreenEdge, /* sea-end anchor1 (mins) */
                                        axx2: Vec<f64>, ayy2: ScreenEdge, /* sea-end anchor2 (maxes) */
                                        pxx1: ShipEnd, pyy1: ShipEnd,       /* ship-end anchor1 */
                                        pxx2: ShipEnd, pyy2: ShipEnd,       /* ship-end anchor2 */
                                        hollow: bool) -> TapeData {
        let x_origin = GLAxis::new_double_origin(&axx1,&axx2, -layer.left(), None,hollow);
        TapeData {
            x_vertex: GLAxis::new_from_double_delta(&pxx1,&pxx2,&x_origin,hollow),
            y_vertex: GLAxis::new_from_double(&ayy1, &pyy1, &ayy2, &pyy2, Some(&x_origin),hollow),
            x_origin,
        }
    }
}

impl GeometryData for TapeData {
    fn iter_screen<'x>(&'x self, stage: &ReadStage) -> Result<Box<dyn Iterator<Item=((f64,f64),(f64,f64))> + 'x>,Message> {
        let x_vertex = self.x_vertex.iter_screen(stage.x())?;
        let x_origin = self.x_origin.iter_paper(stage.x())?;
        let x = x_vertex.zip(x_origin).map(|(s,p)| (s.0+p.0,s.1+p.1));
        let y = self.y_vertex.iter_screen(stage.y())?;
        Ok(Box::new(x.zip(y)))
    }

    fn in_bounds(&self, stage: &ReadStage, mouse: (u32,u32)) -> Result<bool,Message> {
        let mouse = (mouse.0 as f64, mouse.1 as f64);
        let min_x = self.x_vertex.min_screen(stage.x())? - self.x_origin.min_paper(stage.x())?;
        let max_x = self.x_vertex.max_screen(stage.x())? + self.x_origin.max_paper(stage.x())?;
        let min_y = self.y_vertex.min_screen(stage.y())?;
        let max_y = self.y_vertex.max_screen(stage.y())?;
        Ok(!(mouse.0 < min_x || mouse.0 > max_x || mouse.1 < min_y || mouse.1 > max_y))
    }
}

#[derive(Clone)]
pub struct TapeGeometry {
    variety: TapeProgram,
    patina: PatinaProcessName
}

impl TapeGeometry {
    pub(crate) fn new(patina: &PatinaProcessName, variety: &TapeProgram) -> Result<TapeGeometry,Message> {
        Ok(TapeGeometry { variety: variety.clone(), patina: patina.clone() })
    }
}
