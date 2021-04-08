use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProcessName;
use super::super::layers::patina::PatinaProcessName;
use crate::webgl::{ AttribHandle, ProtoProcess, ProcessStanzaElements, Program, ProcessStanzaAddable, GPUSpec, ProgramBuilder };
use peregrine_data::{ScreenEdge, ShipEnd };
use super::super::util::glaxis::GLAxis;
use crate::shape::core::stage::{ ReadStage };
use super::geometrydata::GeometryData;
use crate::util::message::Message;
use web_sys::WebGlRenderingContext;

#[derive(Clone)]
pub struct FixProgram {
    vertexes: AttribHandle,
    signs: AttribHandle,
}

impl FixProgram {
    pub(crate) fn new(builder: &ProgramBuilder) -> Result<FixProgram,Message> {
        Ok(FixProgram {
            vertexes: builder.get_attrib_handle("aVertexPosition")?,
            signs: builder.get_attrib_handle("aSign")?
        })
    }
}

pub struct FixData {
    x: GLAxis,
    y: GLAxis
}

impl FixData {
    pub(crate) fn add_rectangles(sea_x: ScreenEdge, sea_y: ScreenEdge,
                                 ship_x: ShipEnd, ship_y: ShipEnd,
                                 size_x: Vec<f64>, size_y: Vec<f64>, hollow: bool) -> FixData {
        let x = GLAxis::new_from_single(&sea_x,&ship_x,&size_x,None,hollow);
        let y = GLAxis::new_from_single(&sea_y,&ship_y,&size_y,Some(&x),hollow);
        FixData { x, y }
    }

    pub(crate) fn add_stretchtangle(axx1: ScreenEdge, ayy1: ScreenEdge, /* sea-end anchor1 (mins) */
                                    axx2: ScreenEdge, ayy2: ScreenEdge, /* sea-end anchor2 (maxes) */
                                    pxx1: ShipEnd, pyy1: ShipEnd,       /* ship-end anchor1 */
                                    pxx2: ShipEnd, pyy2: ShipEnd,       /* ship-end anchor2 */
                                    hollow: bool) -> FixData {
        let x =  GLAxis::new_from_double(&axx1,&pxx1, &axx2, &pxx2, None,hollow);
        let y = GLAxis::new_from_double(&ayy1,&pyy1, &ayy2, &pyy2, Some(&x),hollow);
        FixData { x, y }
    }
}

impl GeometryData for FixData {
    fn in_bounds(&self, stage: &ReadStage, mouse: (u32,u32)) -> Result<bool,Message> {
        let mouse = (mouse.0 as f64, mouse.1 as f64);
        Ok(!(mouse.0 < self.x.min_screen(stage.x())? || mouse.0 > self.x.max_screen(stage.x())? || 
           mouse.1 < self.y.min_screen(stage.y())? || mouse.1 > self.y.max_screen(stage.y())?))
    }

    fn iter_screen<'x>(&'x self, stage: &ReadStage) -> Result<Box<dyn Iterator<Item=((f64,f64),(f64,f64))> + 'x>,Message> {
        Ok(Box::new(self.x.iter_screen(stage.x())?.zip(self.y.iter_screen(stage.y())?)))
    }
}

#[derive(Clone)]
pub struct FixGeometry {
    variety: FixProgram,
    patina: PatinaProcessName
}

impl FixGeometry {
    pub(crate) fn new(patina: &PatinaProcessName, variety: &FixProgram) -> Result<FixGeometry,Message> {
        Ok(FixGeometry { variety: variety.clone(), patina: patina.clone() })
    }

    pub(crate) fn add(&self, layer: &mut Layer, data: FixData) -> Result<ProcessStanzaElements,Message> {
        let mut elements = data.x.make_elements(layer,&GeometryProcessName::Fix,&self.patina)?;
        elements.add(&self.variety.vertexes,data.x.vec2d(&data.y),2)?;
        elements.add(&self.variety.signs,data.x.signs_2d(&data.y),2)?;
        Ok(elements)
    }
}
