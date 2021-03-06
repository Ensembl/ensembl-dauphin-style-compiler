use std::collections::HashMap;
use peregrine_core::{AnchorPair, SeaEnd, SingleAnchor, ZMenu, ZMenuGenerator};
use crate::shape::core::stage::Stage;
use crate::shape::core::fixgeometry::FixZMenuRectangle;
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
                self.add_region(Box::new(FixZMenuRectangle::new(generator,sea_x,sea_y,ship_x,ship_y,x_size,y_size,allotment)));
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
