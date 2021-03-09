use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProcessName;
use super::super::layers::patina::PatinaProcessName;
use crate::webgl::{ AttribHandle, ProtoProcess, ProcessStanzaElements, Program, ProcessStanzaAddable };
use peregrine_core::{ ShipEnd, ScreenEdge, ZMenuGenerator };
use super::super::util::glaxis::GLAxis;
use super::super::layers::drawingzmenus::{ ZMenuRegion, ZMenuResult };
use super::super::util::arrayutil::{ empty_is };
use crate::shape::core::stage::Stage;

#[derive(Clone)]
pub struct PageProgram {
    vertexes: AttribHandle,
    signs: AttribHandle,
}

impl PageProgram {
    pub(crate) fn new(program: &Program) -> anyhow::Result<PageProgram> {
        Ok(PageProgram {
            vertexes: program.get_attrib_handle("aVertexPosition")?,
            signs: program.get_attrib_handle("aSign")?
        })
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
        PageData {
            x: GLAxis::new_from_single(&sea_x,&ship_x,&size_x,true,hollow),
            y: GLAxis::new_from_single(&ScreenEdge::Min(yy),&ship_y,&size_y,false,hollow)
        }
    }

    pub(crate) fn add_stretchtangle(axx1: ScreenEdge, ayy1: Vec<f64>, /* sea-end anchor1 (mins) */
                                    axx2: ScreenEdge, ayy2: Vec<f64>, /* sea-end anchor2 (maxes) */
                                    pxx1: ShipEnd, pyy1: ShipEnd,      /* ship-end anchor1 */
                                    pxx2: ShipEnd, pyy2: ShipEnd,      /* ship-end anchor2 */
                                    hollow: bool) -> PageData {
        PageData {
            x: GLAxis::new_from_double(&axx1,&pxx1, &axx2, &pxx2, true,hollow),
            y: GLAxis::new_from_double(&ScreenEdge::Min(ayy1),&pyy1, &ScreenEdge::Min(ayy2), &pyy2, false,hollow)
        }
    }
}

#[derive(Clone)]
pub struct PageGeometry {
    variety: PageProgram,
    patina: PatinaProcessName
}

impl PageGeometry {
    pub(crate) fn new(_process: &ProtoProcess, patina: &PatinaProcessName, variety: &PageProgram) -> anyhow::Result<PageGeometry> {
        Ok(PageGeometry { variety: variety.clone(), patina: patina.clone() })
    }

    pub(crate) fn add(&self,layer: &mut Layer, data: PageData) -> anyhow::Result<ProcessStanzaElements> {
        let mut elements = data.y.make_elements(layer, &GeometryProcessName::Page,&self.patina)?;
        elements.add(&self.variety.vertexes,data.x.vec2d(&data.y))?;
        elements.add(&self.variety.signs,data.x.signs_2d(&data.y))?;
        Ok(elements)
    }
}

pub struct PageZMenuRectangle {
    zmenu: ZMenuGenerator,
    data: PageData,
    allotment: Vec<String>
}

impl PageZMenuRectangle {
    pub fn new(zmenu: ZMenuGenerator, data: PageData, allotment: Vec<String>) -> PageZMenuRectangle {
        PageZMenuRectangle {
            zmenu, data,
            allotment: empty_is(allotment,"".to_string())
        }
    }
}

impl ZMenuRegion for PageZMenuRectangle {
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
