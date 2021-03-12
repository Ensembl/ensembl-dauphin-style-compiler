use std::collections::HashMap;
use peregrine_data::{AnchorPair, SeaEnd, SeaEndPair, SingleAnchor, ZMenu, ZMenuGenerator};
use crate::shape::core::stage::{ ReadStage };
use crate::shape::core::fixgeometry::{ FixData };
use crate::shape::core::pagegeometry::{ PageData };
use crate::shape::core::pingeometry::{ PinData };
use crate::shape::core::tapegeometry::{ TapeData };
use peregrine_data::ZMenuFixed;
use crate::shape::core::geometrydata::{ GeometryData, ZMenuRectangle };
use super::super::layers::layer::{ Layer };

pub struct ZMenuResult {
    menu: ZMenuFixed,
    allotment: String
}

impl ZMenuResult {
    pub fn new(menu: ZMenuFixed, allotment: &str) -> ZMenuResult {
        ZMenuResult {
            menu,
            allotment: allotment.to_string()
        }
    }
}

pub struct ZMenuEvent {
    menu: ZMenuFixed,
    pixel: (u32,u32),
    bp: (f64,u32), // TODO allotment y
    allotment: String // TODO allotments
}

pub struct DrawingZMenusBuilder {
    entries: Vec<ZMenuRectangle>
}

impl DrawingZMenusBuilder {
    pub(crate) fn new() -> DrawingZMenusBuilder {
        DrawingZMenusBuilder {
            entries: vec![]
        }
    }

    fn add_region(&mut self, generator: ZMenuGenerator, region: Box<dyn GeometryData>, allotment: Vec<String>) {
        self.entries.push(ZMenuRectangle::new(generator,region,allotment));
    }

    pub(crate) fn add_rectangle(&mut self, layer: &Layer, zmenu: ZMenu, values: HashMap<String,Vec<String>>, anchor: SingleAnchor, allotment: Vec<String>, x_size: Vec<f64>, y_size: Vec<f64>) {
        let generator = ZMenuGenerator::new(&zmenu,&values);
        let region : Box<dyn GeometryData> = match ((anchor.0).0,(anchor.0).1,(anchor.1).0,(anchor.1).1) {
            (SeaEnd::Screen(sea_x),ship_x,SeaEnd::Screen(sea_y),ship_y) => {
                Box::new(FixData::add_rectangles(sea_x,sea_y,ship_x,ship_y,x_size,y_size,false))
            },
            (SeaEnd::Screen(sea_x),ship_x,SeaEnd::Paper(yy),ship_y) => {
                Box::new(PageData::add_rectangles(sea_x,yy,ship_x,ship_y,x_size,y_size,false))
            },
            (SeaEnd::Paper(xx),ship_x,SeaEnd::Paper(yy),ship_y) => {
                Box::new(PinData::add_rectangles(layer,xx,yy,ship_x,ship_y,x_size,y_size,false))
            },
            (SeaEnd::Paper(xx),ship_x,SeaEnd::Screen(sea_y),ship_y) => {
                Box::new(TapeData::add_rectangles(layer,xx,sea_y,ship_x,ship_y,x_size,y_size,false))
            },
        };
        self.add_region(generator,region,allotment);
    }

    pub(crate) fn add_stretchtangle(&mut self, layer: &Layer, zmenu: ZMenu, values: HashMap<String,Vec<String>>, anchors: AnchorPair, allotment: Vec<String>) {
        let generator = ZMenuGenerator::new(&zmenu,&values);
        let anchors_x = anchors.0;
        let anchors_y = anchors.1;
        let anchor_sea_x = anchors_x.0;
        let pxx1 = anchors_x.1;
        let pxx2 = anchors_x.2;
        let anchor_sea_y = anchors_y.0;
        let pyy1 = anchors_y.1;
        let pyy2 = anchors_y.2;
        let region : Box<dyn GeometryData> = match (anchor_sea_x,anchor_sea_y) {
            (SeaEndPair::Screen(axx1,axx2),SeaEndPair::Screen(ayy1,ayy2)) => {
                Box::new(FixData::add_stretchtangle(axx1,ayy1,axx2,ayy2,pxx1,pyy1,pxx2,pyy2,false))
            },
            (SeaEndPair::Screen(axx1,axx2),SeaEndPair::Paper(ayy1,ayy2)) => {
                Box::new(PageData::add_stretchtangle(axx1,ayy1,axx2,ayy2,pxx1,pyy1,pxx2,pyy2,false))
            },
            (SeaEndPair::Paper(axx1,axx2),SeaEndPair::Paper(ayy1,ayy2)) => {
                Box::new(PinData::add_stretchtangle(layer,axx1,ayy1,axx2,ayy2,pxx1,pyy1,pxx2,pyy2,false))
            },
            (SeaEndPair::Paper(axx1,axx2),SeaEndPair::Screen(ayy1,ayy2)) => {
                Box::new(TapeData::add_stretchtangle(layer,axx1,ayy1,axx2,ayy2,pxx1,pyy1,pxx2,pyy2,false))
            }
        };
        self.add_region(generator,region,allotment);
    }

    pub(crate) fn build(mut self) -> DrawingZMenus {
        self.entries.reverse(); // we match top-down!
        DrawingZMenus::new(self.entries)
    }
}

pub struct DrawingZMenus {
    entries: Vec<ZMenuRectangle>
}

impl DrawingZMenus {
    fn new(entries: Vec<ZMenuRectangle>) -> DrawingZMenus {
        DrawingZMenus {
            entries
        }
    }

    pub(crate) fn intersects(&self, stage: &ReadStage, mouse: (u32,u32)) -> anyhow::Result<Option<ZMenuEvent>> {
        for entry in &self.entries {
            if let Some(result) = entry.intersects(stage,mouse)? {
                return Ok(Some(ZMenuEvent {
                    menu: result.menu,
                    pixel: mouse,
                    bp: (stage.x().position()?,0), // TODO allotment y
                    allotment: result.allotment
                }));
            }
        }
        Ok(None)
    }

    pub(crate) fn intersects_fast(&self, stage: &ReadStage, mouse: (u32,u32)) -> anyhow::Result<bool> {
        for entry in &self.entries {
            if entry.intersects_fast(stage,mouse)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}