use std::collections::HashMap;
use peregrine_core::{AnchorPair, SeaEnd, SeaEndPair, SingleAnchor, ZMenu, ZMenuGenerator};
use crate::shape::core::stage::Stage;
use crate::shape::core::fixgeometry::FixZMenuRectangle;
use super::super::core::fixgeometry::FixData;
use peregrine_core::ZMenuFixed;

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

pub trait ZMenuRegion {
    fn intersects(&self, stage: &Stage, mouse: (u32,u32)) -> anyhow::Result<Option<ZMenuResult>>;
}

struct ZMenuEntry {
    region: Box<dyn ZMenuRegion>
}

pub struct DrawingZMenusBuilder {
    entries: Vec<ZMenuEntry>
}

impl DrawingZMenusBuilder {
    pub(crate) fn new() -> DrawingZMenusBuilder {
        DrawingZMenusBuilder {
            entries: vec![]
        }
    }

    fn add_region(&mut self, region: Box<dyn ZMenuRegion>) {
        let entry = ZMenuEntry {
            region
        };
        self.entries.push(entry);
    }

    pub(crate) fn add_rectangle(&mut self, zmenu: ZMenu, values: HashMap<String,Vec<String>>, anchor: SingleAnchor, allotment: Vec<String>, x_size: Vec<f64>, y_size: Vec<f64>) {
        let generator = ZMenuGenerator::new(&zmenu,&values);
        match ((anchor.0).0,(anchor.0).1,(anchor.1).0,(anchor.1).1) {
            (SeaEnd::Screen(sea_x),ship_x,SeaEnd::Screen(sea_y),ship_y) => {
                let fix_data = FixData::add_rectangles(sea_x,sea_y,ship_x,ship_y,x_size,y_size,false);
                self.add_region(Box::new(FixZMenuRectangle::new(generator,fix_data,allotment)));
            },
            _ => {}
            /*
            (SeaEnd::Paper(xx),ship_x,SeaEnd::Paper(yy),ship_y) => {
                Ok((layer.get_pin(skin)?.add_rectangles(layer,xx,yy,ship_x,ship_y,x_size,y_size,hollow)?,GeometryProcessName::Pin))
            },
            (SeaEnd::Paper(xx),ship_x,SeaEnd::Screen(sea_y),ship_y) => {
                Ok((layer.get_tape(skin)?.add_rectangles(layer,xx,sea_y,ship_x,ship_y,x_size,y_size,hollow)?,GeometryProcessName::Tape))         
            },
            (SeaEnd::Screen(sea_x),ship_x,SeaEnd::Paper(yy),ship_y) => {
                Ok((layer.get_page(skin)?.add_rectangles(layer,sea_x,yy,ship_x,ship_y,x_size,y_size,hollow)?,GeometryProcessName::Page))
            }
            */
        }
    }

    pub(crate) fn add_stretchtangle(&mut self, zmenu: ZMenu, values: HashMap<String,Vec<String>>, anchors: AnchorPair, allotment: Vec<String>) {
        let generator = ZMenuGenerator::new(&zmenu,&values);
        let anchors_x = anchors.0;
        let anchors_y = anchors.1;
        let anchor_sea_x = anchors_x.0;
        let pxx1 = anchors_x.1;
        let pxx2 = anchors_x.2;
        let anchor_sea_y = anchors_y.0;
        let pyy1 = anchors_y.1;
        let pyy2 = anchors_y.2;
        match (anchor_sea_x,anchor_sea_y) {
            (SeaEndPair::Screen(axx1,axx2),SeaEndPair::Screen(ayy1,ayy2)) => {
                let fix_data = FixData::add_stretchtangle(axx1,ayy1,axx2,ayy2,pxx1,pyy1,pxx2,pyy2,false);
                self.add_region(Box::new(FixZMenuRectangle::new(generator,fix_data,allotment)))
            },
            _ => {}
            /* 
            (SeaEndPair::Paper(axx1,axx2),SeaEndPair::Paper(ayy1,ayy2)) => {
                Ok((layer.get_pin(skin)?.add_stretchtangle(layer,axx1,ayy1,axx2,ayy2,pxx1,pyy1,pxx2,pyy2,hollow)?,GeometryProcessName::Pin))
            },
            (SeaEndPair::Paper(axx1,axx2),SeaEndPair::Screen(ayy1,ayy2)) => {
                Ok((layer.get_tape(skin)?.add_stretchtangle(layer,axx1,ayy1,axx2,ayy2,pxx1,pyy1,pxx2,pyy2,hollow)?,GeometryProcessName::Tape))
            },
            (SeaEndPair::Screen(axx1,axx2),SeaEndPair::Paper(ayy1,ayy2)) => {
                Ok((layer.get_page(skin)?.add_stretchtangle(layer,axx1,ayy1,axx2,ayy2,pxx1,pyy1,pxx2,pyy2,hollow)?,GeometryProcessName::Page))
            }
            */
        }
    
    }

    pub(crate) fn build(self) -> DrawingZMenus {
        DrawingZMenus::new(self.entries)
    }
}

// TODO pointer (y efficiency, bounding box)

pub struct DrawingZMenus {
    entries: Vec<ZMenuEntry>
}

impl DrawingZMenus {
    fn new(entries: Vec<ZMenuEntry>) -> DrawingZMenus {
        DrawingZMenus {
            entries
        }
    }

    fn intersects(&self, stage: &Stage, mouse: (u32,u32)) -> anyhow::Result<Option<ZMenuEvent>> {
        for entry in &self.entries {
            if let Some(result) = entry.region.intersects(stage,mouse)? {
                return Ok(Some(ZMenuEvent {
                    menu: result.menu,
                    pixel: mouse,
                    bp: (stage.x_position()?,0), // TODO allotment y
                    allotment: result.allotment
                }));
            }
        }
        Ok(None)
    }
}
