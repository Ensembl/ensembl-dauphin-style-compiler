use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProcessName;
use super::super::layers::patina::PatinaProcessName;
use crate::webgl::{ AttribHandle, ProtoProcess, ProcessStanzaElements, Program, ProcessStanzaAddable };
use peregrine_core::{ScreenEdge, ShipEnd, ZMenuGenerator };
use super::super::util::arrayutil::{ empty_is };
use super::super::util::glaxis::GLAxis;
use super::super::layers::drawingzmenus::{ ZMenuRegion, ZMenuResult };
use crate::shape::core::stage::Stage;

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

    fn add(&self, layer: &mut Layer, x_axis: GLAxis, y_axis: GLAxis) -> anyhow::Result<ProcessStanzaElements> {
        let mut elements = x_axis.make_elements(layer,&GeometryProcessName::Fix,&self.patina)?;
        elements.add(&self.variety.vertexes,x_axis.vec2d(&y_axis))?;
        elements.add(&self.variety.signs,x_axis.signs_2d(&y_axis))?;
        Ok(elements)
    }

    pub(crate) fn add_rectangles(&self, layer: &mut Layer,
                                        sea_x: ScreenEdge, sea_y: ScreenEdge,
                                        ship_x: ShipEnd, ship_y: ShipEnd,
                                        size_x: Vec<f64>, size_y: Vec<f64>, hollow: bool) -> anyhow::Result<ProcessStanzaElements> {
        let x_axis = GLAxis::new_from_single(&sea_x,&ship_x,&size_x,true,hollow);
        let y_axis = GLAxis::new_from_single(&sea_y,&ship_y,&size_y,false,hollow);
        self.add(layer,x_axis,y_axis)
    }

    pub(crate) fn add_stretchtangle(&self, layer: &mut Layer, 
                                        axx1: ScreenEdge, ayy1: ScreenEdge, /* sea-end anchor1 (mins) */
                                        axx2: ScreenEdge, ayy2: ScreenEdge, /* sea-end anchor2 (maxes) */
                                        pxx1: ShipEnd, pyy1: ShipEnd,       /* ship-end anchor1 */
                                        pxx2: ShipEnd, pyy2: ShipEnd,       /* ship-end anchor2 */
                                        hollow: bool) -> anyhow::Result<ProcessStanzaElements> {
        let x_axis = GLAxis::new_from_double(&axx1,&pxx1, &axx2, &pxx2, true,hollow);
        let y_axis = GLAxis::new_from_double(&ayy1,&pyy1, &ayy2, &pyy2, false,hollow);
        self.add(layer,x_axis,y_axis)
    }
}

pub struct FixZMenuRectangle {
    zmenu: ZMenuGenerator,
    x: GLAxis,
    y: GLAxis,
    allotment: Vec<String>
}

impl FixZMenuRectangle {
    pub fn new(zmenu: ZMenuGenerator, sea_x: ScreenEdge, sea_y: ScreenEdge,
                      ship_x: ShipEnd, ship_y: ShipEnd,
                      size_x: Vec<f64>, size_y: Vec<f64>, allotment: Vec<String>) -> FixZMenuRectangle {
        let x = GLAxis::new_from_single(&sea_x, &ship_x, &size_x, true,false);
        let y = GLAxis::new_from_single(&sea_y, &ship_y, &size_y, false,false);
        FixZMenuRectangle {
            zmenu, x, y, 
            allotment: empty_is(allotment,"".to_string())
        }
    }
}

impl ZMenuRegion for FixZMenuRectangle {
    fn intersects(&self, stage: &Stage, mouse: (u32,u32)) -> anyhow::Result<Option<ZMenuResult>> {
        let mouse = (mouse.0 as f64, mouse.1 as f64);
        let size = stage.size()?;
        if mouse.0 < self.x.min_screen(size.0) || mouse.0 > self.x.max_screen(size.0) || 
            mouse.1 < self.y.min_screen(size.1) || mouse.1 > self.y.max_screen(size.1) {
                return Ok(None);
        }
        let looper = self.x.iter_screen(size.0).zip(self.y.iter_screen(size.1)).zip(self.allotment.iter().cycle());
        for (index,((x,y),allotment)) in looper.enumerate() {
            if mouse.0 < x.0 || mouse.0 > x.1 || mouse.1 < y.0 || mouse.1 > y.1 { continue; }
            return Ok(Some(ZMenuResult::new(self.zmenu.make_proxy(index).value(),allotment)))
        }
        Ok(None)
    }
}
