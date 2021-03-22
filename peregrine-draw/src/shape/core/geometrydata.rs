use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProcessName;
use super::super::layers::patina::PatinaProcessName;
use crate::webgl::{ AttribHandle, ProtoProcess, ProcessStanzaElements, Program, ProcessStanzaAddable };
use peregrine_data::{ScreenEdge, ShipEnd, ZMenuGenerator };
use super::super::util::arrayutil::{ empty_is };
use super::super::util::glaxis::GLAxis;
use super::super::layers::drawingzmenus::{ ZMenuResult };
use crate::shape::core::stage::{ ReadStage, ReadStageAxis };
use crate::util::message::Message;

pub trait GeometryData {
    fn iter_screen<'x>(&'x self, stage: &ReadStage) -> Result<Box<Iterator<Item=((f64,f64),(f64,f64))> + 'x>,Message>;
    fn in_bounds(&self, stage: &ReadStage, mouse: (u32,u32)) -> Result<bool,Message>;
}

pub struct ZMenuRectangle {
    zmenu: ZMenuGenerator,
    data: Box<dyn GeometryData>,
    allotment: Vec<String>
}

impl ZMenuRectangle {
    pub fn new(zmenu: ZMenuGenerator, data: Box<dyn GeometryData>, allotment: Vec<String>) -> ZMenuRectangle {
        ZMenuRectangle {
            zmenu, data,
            allotment: empty_is(allotment,"".to_string())
        }
    }

    fn intersects_test<'t>(&'t self, stage: &ReadStage, mouse: (u32,u32)) -> Result<Option<(usize,&'t str)>,Message> {
        if !self.data.in_bounds(stage,mouse)? {
            return Ok(None);
        }
        let mouse = (mouse.0 as f64, mouse.1 as f64);
        let looper = self.data.iter_screen(stage)?.zip(self.allotment.iter().cycle());
        for (index,((x,y),allotment)) in looper.enumerate() {
            if mouse.0 < x.0 || mouse.0 > x.1 || mouse.1 < y.0 || mouse.1 > y.1 { continue; }
            return Ok(Some((index,allotment)))
        }
        Ok(None)
    }

    pub(crate) fn intersects_fast(&self, stage: &ReadStage, mouse: (u32,u32)) -> Result<bool,Message> {
        Ok(self.intersects_test(stage,mouse)?.is_some())
    }

    pub(crate) fn intersects(&self, stage: &ReadStage, mouse: (u32,u32)) -> Result<Option<ZMenuResult>,Message> {
        Ok(self.intersects_test(stage,mouse)?.map(|(index,allotment)| {
            ZMenuResult::new(self.zmenu.make_proxy(index).value(),allotment)
        }))
    }
}
