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

pub struct FixData {
    x: GLAxis,
    y: GLAxis
}

impl FixData {
    pub(crate) fn add_rectangles(sea_x: ScreenEdge, sea_y: ScreenEdge,
                                 ship_x: ShipEnd, ship_y: ShipEnd,
                                 size_x: Vec<f64>, size_y: Vec<f64>, hollow: bool) -> FixData {
        FixData {
            x: GLAxis::new_from_single(&sea_x,&ship_x,&size_x,true,hollow),
            y: GLAxis::new_from_single(&sea_y,&ship_y,&size_y,false,hollow)
        }
    }

    pub(crate) fn add_stretchtangle(axx1: ScreenEdge, ayy1: ScreenEdge, /* sea-end anchor1 (mins) */
                                    axx2: ScreenEdge, ayy2: ScreenEdge, /* sea-end anchor2 (maxes) */
                                    pxx1: ShipEnd, pyy1: ShipEnd,       /* ship-end anchor1 */
                                    pxx2: ShipEnd, pyy2: ShipEnd,       /* ship-end anchor2 */
                                    hollow: bool) -> FixData {
        FixData {
            x: GLAxis::new_from_double(&axx1,&pxx1, &axx2, &pxx2, true,hollow),
            y: GLAxis::new_from_double(&ayy1,&pyy1, &ayy2, &pyy2, false,hollow)
        }
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

    pub(crate) fn add(&self, layer: &mut Layer, data: FixData) -> anyhow::Result<ProcessStanzaElements> {
        let mut elements = data.x.make_elements(layer,&GeometryProcessName::Fix,&self.patina)?;
        elements.add(&self.variety.vertexes,data.x.vec2d(&data.y))?;
        elements.add(&self.variety.signs,data.x.signs_2d(&data.y))?;
        Ok(elements)
    }
}

pub struct FixZMenuRectangle {
    zmenu: ZMenuGenerator,
    data: FixData,
    allotment: Vec<String>
}

impl FixZMenuRectangle {
    pub fn new(zmenu: ZMenuGenerator, data: FixData, allotment: Vec<String>) -> FixZMenuRectangle {
        FixZMenuRectangle {
            zmenu, data,
            allotment: empty_is(allotment,"".to_string())
        }
    }
}

impl ZMenuRegion for FixZMenuRectangle {
    fn intersects(&self, stage: &Stage, mouse: (u32,u32)) -> anyhow::Result<Option<ZMenuResult>> {
        let mouse = (mouse.0 as f64, mouse.1 as f64);
        let size = stage.size()?;
        if mouse.0 < self.data.x.min_screen(size.0) || mouse.0 > self.data.x.max_screen(size.0) || 
            mouse.1 < self.data.y.min_screen(size.1) || mouse.1 > self.data.y.max_screen(size.1) {
                return Ok(None);
        }
        let looper = self.data.x.iter_screen(size.0).zip(self.data.y.iter_screen(size.1)).zip(self.allotment.iter().cycle());
        for (index,((x,y),allotment)) in looper.enumerate() {
            if mouse.0 < x.0 || mouse.0 > x.1 || mouse.1 < y.0 || mouse.1 > y.1 { continue; }
            return Ok(Some(ZMenuResult::new(self.zmenu.make_proxy(index).value(),allotment)))
        }
        Ok(None)
    }
}
