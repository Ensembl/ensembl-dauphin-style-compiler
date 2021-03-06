use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProcessName;
use super::super::layers::patina::PatinaProcessName;
use crate::webgl::{ AttribHandle, ProtoProcess, ProcessStanzaElements, Program, ProcessStanzaAddable };
use peregrine_core::{ScreenEdge, ShipEnd, ZMenuGenerator };
use super::arrayutil::{ empty_is, make_rect_elements, repeat, interleave_rect_x, calculate_vertex, sea_sign, calculate_stretch_vertex, interleave_pair_count };
use super::super::layers::drawingzmenus::{ ZMenuRegion, ZMenuResult };
use crate::shape::core::stage::Stage;
use crate::looper;

const HOLLOW_WIDTH : f64 = 1.; // XXX

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

    pub(crate) fn add_rectangles(&self, layer: &mut Layer,
                                        sea_x: ScreenEdge, sea_y: ScreenEdge,
                                        ship_x: ShipEnd, ship_y: ShipEnd,
                                        size_x: Vec<f64>, size_y: Vec<f64>, hollow: bool) -> anyhow::Result<ProcessStanzaElements> {
        let len = sea_x.len();
        let mut elements = make_rect_elements(layer,&GeometryProcessName::Fix,&self.patina,len,hollow)?;
        let x1 = calculate_vertex(&sea_x,&ship_x,&size_x,false);
        let x2 = calculate_vertex(&sea_x,&ship_x,&size_x,true);
        let y1 = calculate_vertex(&sea_y,&ship_y,&size_y,false);
        let y2 = calculate_vertex(&sea_y,&ship_y,&size_y,true);
        let signs = interleave_pair_count(sea_sign(&sea_x),sea_sign(&sea_y),len*if hollow {8} else {4})?;
        let vertexes = interleave_rect_x(&x1,&y1,&x2,&y2,if hollow {Some(HOLLOW_WIDTH)} else {None});
        elements.add(&self.variety.vertexes,vertexes)?; /* 8n */
        elements.add(&self.variety.signs,signs)?; /* 8n */
        Ok(elements)
    }

    pub(crate) fn add_stretchtangle(&self, layer: &mut Layer, 
                                        axx1: ScreenEdge, ayy1: ScreenEdge, /* sea-end anchor1 (mins) */
                                        axx2: ScreenEdge, ayy2: ScreenEdge, /* sea-end anchor2 (maxes) */
                                        pxx1: ShipEnd, pyy1: ShipEnd,       /* ship-end anchor1 */
                                        pxx2: ShipEnd, pyy2: ShipEnd,       /* ship-end anchor2 */
                                        hollow: bool) -> anyhow::Result<ProcessStanzaElements> {
        let len = axx1.len();
        let mut elements = make_rect_elements(layer,&GeometryProcessName::Fix,&self.patina,len,hollow)?;
        let x1 = calculate_stretch_vertex(&axx1,&pxx1);
        let x2 = calculate_stretch_vertex(&axx2,&pxx2);
        let y1 = calculate_stretch_vertex(&ayy1,&pyy1);
        let y2 = calculate_stretch_vertex(&ayy2,&pyy2);
        let vertexes = interleave_rect_x(&x1,&y1,&x2,&y2,if hollow {Some(HOLLOW_WIDTH)} else {None});
        let sx1 = sea_sign(&axx1);
        let sx2 = sea_sign(&axx2);
        let sy1 = sea_sign(&ayy1);
        let sy2 = sea_sign(&ayy2);
        let signs = if hollow {
            repeat(&[sx1,sy1,sx1,sy1,  sx1,sy2,sx1,sy2,   sx2,sy1,sx2,sy1,   sx2,sy2,sx2,sy2],len)
        } else {
            repeat(&[sx1,sy1,  sx1,sy2,   sx2,sy1,   sx2,sy2],len)
        };
        elements.add(&self.variety.vertexes,vertexes)?; /* 8n */
        elements.add(&self.variety.signs,signs)?; /* 8n */
        Ok(elements)
    }
}

pub struct FixZMenuRectangle {
    zmenu: ZMenuGenerator,
    min: (f64,f64),
    max: (f64,f64),
    x1: Vec<f64>,
    y1: Vec<f64>,
    x2: Vec<f64>,
    y2: Vec<f64>,
    allotment: Vec<String>,
    x_sign: f64,
    y_sign: f64
}

looper!(FixZMenuRectangleLoop,FixZMenuRectangle,{x1,f64},[{y1,f64},{x2,f64},{y2,f64},{allotment,String}]);

impl FixZMenuRectangle {
    pub fn new(zmenu: ZMenuGenerator, sea_x: ScreenEdge, sea_y: ScreenEdge,
                      ship_x: ShipEnd, ship_y: ShipEnd,
                      size_x: Vec<f64>, size_y: Vec<f64>, allotment: Vec<String>) -> FixZMenuRectangle {
        let x1 = empty_is(calculate_vertex(&sea_x,&ship_x,&size_x,false),0.);
        let x2 = empty_is(calculate_vertex(&sea_x,&ship_x,&size_x,true),0.);
        let y1 = empty_is(calculate_vertex(&sea_y,&ship_y,&size_y,false),0.);
        let y2 = empty_is(calculate_vertex(&sea_y,&ship_y,&size_y,true),0.);
        let min = (
            x1.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
            y1.iter().fold(f64::INFINITY, |a, &b| a.min(b)));
        let max = (
            x1.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
            y1.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));    
            FixZMenuRectangle {
            zmenu, x1, x2, y1, y2, min, max, x_sign: sea_sign(&sea_x), y_sign: sea_sign(&sea_y), 
            allotment: empty_is(allotment,"".to_string())
        }
    }
}

impl ZMenuRegion for FixZMenuRectangle {
    fn intersects(&self, stage: &Stage, mouse: (u32,u32)) -> anyhow::Result<Option<ZMenuResult>> {
        let mouse = (mouse.0 as f64, mouse.1 as f64);
        let size = stage.size()?;
        let x = if self.x_sign < 0. { size.0 - mouse.0 } else { mouse.0 };
        let y = if self.y_sign < 0. { size.1 - mouse.1 } else { mouse.1 };
        if x < self.min.0 || x > self.max.0 || y < self.min.1 || y > self.max.1 { return Ok(None); } 
        let looper = FixZMenuRectangleLoop::new(self);
        for (index,(x1,y1,x2,y2,allotment)) in looper.enumerate() {
            if x < *x1 || x > *x2 || y < *y1 || y > *y2 { continue; }
            return Ok(Some(ZMenuResult::new(self.zmenu.make_proxy(index).value(),allotment)))
        }
        Ok(None)
    }
}
