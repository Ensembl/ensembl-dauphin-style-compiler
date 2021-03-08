use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProcessName;
use super::super::layers::patina::PatinaProcessName;
use crate::webgl::{ AttribHandle, ProtoProcess, ProcessStanzaElements, Program, ProcessStanzaAddable };
use peregrine_core::{ ShipEnd, ScreenEdge };
use super::super::util::glaxis::GLAxis;

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

#[derive(Clone)]
pub struct PageGeometry {
    variety: PageProgram,
    patina: PatinaProcessName
}

impl PageGeometry {
    pub(crate) fn new(_process: &ProtoProcess, patina: &PatinaProcessName, variety: &PageProgram) -> anyhow::Result<PageGeometry> {
        Ok(PageGeometry { variety: variety.clone(), patina: patina.clone() })
    }

    fn add(&self,layer: &mut Layer, x_axis: GLAxis, y_axis: GLAxis) -> anyhow::Result<ProcessStanzaElements> {
        let mut elements = y_axis.make_elements(layer, &GeometryProcessName::Tape,&self.patina)?;
        elements.add(&self.variety.vertexes,x_axis.vec2d(&y_axis))?;
        elements.add(&self.variety.signs,x_axis.signs_2d(&y_axis))?;
        Ok(elements)
    }

    pub(crate) fn add_rectangles(&self, layer: &mut Layer,
                                        sea_x: ScreenEdge, yy: Vec<f64>,
                                        ship_x: ShipEnd, ship_y: ShipEnd,
                                        size_x: Vec<f64>, size_y: Vec<f64>, hollow: bool) -> anyhow::Result<ProcessStanzaElements> {
        let x_axis = GLAxis::new_from_single(&sea_x,&ship_x,&size_x,false,hollow);
        let y_axis = GLAxis::new_from_single(&ScreenEdge::Min(yy),&ship_y,&size_y,true,hollow);
        self.add(layer,x_axis,y_axis)
    }

    pub(crate) fn add_stretchtangle(&self, layer: &mut Layer, 
                axx1: ScreenEdge, ayy1: Vec<f64>, /* sea-end anchor1 (mins) */
                axx2: ScreenEdge, ayy2: Vec<f64>, /* sea-end anchor2 (maxes) */
                pxx1: ShipEnd, pyy1: ShipEnd,      /* ship-end anchor1 */
                pxx2: ShipEnd, pyy2: ShipEnd,      /* ship-end anchor2 */
                hollow: bool        ) -> anyhow::Result<ProcessStanzaElements> {
        let x_axis = GLAxis::new_from_double(&axx1,&pxx1,&axx2,&pxx2,false,hollow);
        let y_axis = GLAxis::new_from_double(&ScreenEdge::Min(ayy1),&pyy1,&ScreenEdge::Min(ayy2),&pyy2,true,hollow);
        self.add(layer,x_axis,y_axis)
    }
}
