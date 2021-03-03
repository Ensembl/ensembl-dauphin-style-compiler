use std::collections::HashMap;
use peregrine_core::{ SingleAnchor, AnchorPair, ZMenu };

pub struct DrawingZMenusBuilder {
}

impl DrawingZMenusBuilder {
    pub(crate) fn new() -> DrawingZMenusBuilder {
        DrawingZMenusBuilder {

        }
    }

    pub(crate) fn add_rectangle(&mut self, zmenu: ZMenu, values: HashMap<String,Vec<String>>, anchor: SingleAnchor, allotment: Vec<String>, x_size: Vec<f64>, y_size: Vec<f64>) {
        
        match ((anchor.0).0,(anchor.0).1,(anchor.1).0,(anchor.1).1) {
            _ => {}
            /*
            (SeaEnd::Paper(xx),ship_x,SeaEnd::Paper(yy),ship_y) => {
                Ok((layer.get_pin(skin)?.add_rectangles(layer,xx,yy,ship_x,ship_y,x_size,y_size,hollow)?,GeometryProcessName::Pin))
            },
            (SeaEnd::Screen(sea_x),ship_x,SeaEnd::Screen(sea_y),ship_y) => {
                Ok((layer.get_fix(skin)?.add_rectangles(layer,sea_x,sea_y,ship_x,ship_y,x_size,y_size,hollow)?,GeometryProcessName::Fix))
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

    pub(crate) fn build(&mut self) -> DrawingZMenus {
        DrawingZMenus::new()
    }
}

pub struct DrawingZMenus {

}

impl DrawingZMenus {
    fn new() -> DrawingZMenus {
        DrawingZMenus {

        }
    }
}