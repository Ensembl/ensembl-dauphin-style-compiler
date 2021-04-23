use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProgramName;
use super::super::layers::patina::PatinaProcessName;
use crate::webgl::{ AttribHandle, ProcessBuilder, ProcessStanzaElements, Program, ProcessStanzaAddable, GPUSpec,ProgramBuilder };
use peregrine_data::{ ShipEnd, ScreenEdge };
use super::super::util::glaxis::GLAxis;
use crate::stage::stage::{ ReadStage };
use super::geometrydata::GeometryData;
use crate::util::message::Message;
use web_sys::WebGlRenderingContext;

#[derive(Clone)]
pub struct PageProgram {
    vertexes: AttribHandle,
    signs: AttribHandle,
}

impl PageProgram {
    pub(crate) fn new(builder: &ProgramBuilder) -> Result<PageProgram,Message> {
        Ok(PageProgram {
            vertexes: builder.get_attrib_handle("aVertexPosition")?,
            signs: builder.get_attrib_handle("aSign")?
        })
    }

    pub(crate) fn add(&self,process: &mut ProcessBuilder, data: PageData) -> Result<ProcessStanzaElements,Message> {
        let mut elements = data.y.make_elements(process)?;
        elements.add(&self.vertexes,data.x.vec2d(&data.y),2)?;
        elements.add(&self.signs,data.x.signs_2d(&data.y),2)?;
        Ok(elements)
    }
}

pub struct PageData {
    x: GLAxis,
    y: GLAxis
}

impl PageData {
    pub(crate) fn add_rectangles(sea_x: ScreenEdge, yy: Vec<f64>,
                                 ship_x: ShipEnd, ship_y: ShipEnd,
                                 size_x: Vec<f64>, size_y: Vec<f64>, hollow: bool) -> PageData {
        let y = GLAxis::new_from_single(&ScreenEdge::Min(yy),&ship_y,&size_y,None,hollow);
        let x = GLAxis::new_from_single(&sea_x,&ship_x,&size_x,Some(&y),hollow);
        PageData { x,y }
    }

    pub(crate) fn add_stretchtangle(axx1: ScreenEdge, ayy1: Vec<f64>, /* sea-end anchor1 (mins) */
                                    axx2: ScreenEdge, ayy2: Vec<f64>, /* sea-end anchor2 (maxes) */
                                    pxx1: ShipEnd, pyy1: ShipEnd,      /* ship-end anchor1 */
                                    pxx2: ShipEnd, pyy2: ShipEnd,      /* ship-end anchor2 */
                                    hollow: bool) -> PageData {
        let y = GLAxis::new_from_double(&ScreenEdge::Min(ayy1),&pyy1, &ScreenEdge::Min(ayy2), &pyy2, None,hollow);
        let x = GLAxis::new_from_double(&axx1,&pxx1, &axx2, &pxx2, Some(&y),hollow);
        PageData { x,y }
    }
}

impl GeometryData for PageData {
    fn iter_screen<'x>(&'x self, stage: &ReadStage) -> Result<Box<dyn Iterator<Item=((f64,f64),(f64,f64))> + 'x>,Message> {
        Ok(Box::new(self.x.iter_screen(stage.x())?.zip(self.y.iter_screen(stage.y())?)))
    }

    fn in_bounds(&self, stage: &ReadStage, mouse: (u32,u32)) -> Result<bool,Message> {
        let mouse = (mouse.0 as f64, mouse.1 as f64);
        Ok(!(mouse.0 < self.x.min_screen(stage.x())? || mouse.0 > self.x.max_screen(stage.x())? || 
           mouse.1 < self.y.min_screen(stage.y())? || mouse.1 > self.y.max_screen(stage.y())?))
    }
}
