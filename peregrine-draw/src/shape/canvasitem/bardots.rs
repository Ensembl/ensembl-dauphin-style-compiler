use peregrine_data::DirectColour;
use peregrine_toolkit::{error::Error};
use crate::webgl::canvas::htmlcanvas::canvasinuse::CanvasAndContext;

use super::heraldry::{HeraldryHandleType, HeraldryScale};

/* A bar indicates a certain number of stripes and strecthes.
 * Dots represent a certain dot length and repeat.
 */

const BAR_WIDTH : u32 = 4;

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Hash,Clone)]
enum Variety {
    Bar,
    Dots
}

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Hash,Clone)]
pub(crate) struct HeraldryBarDots {
    col_a: DirectColour,
    col_b: DirectColour,
    prop: u32,
    len: u32,
    dir: bool,
    variety: Variety
}

fn at_least_one(x: &mut (u32,u32)) {
    x.0 = x.0.max(1);
    x.1 = x.1.max(1);
}

fn sanitise_mult(mult: f64) -> f64 {
    (1 << (mult.log2().round() as u32)) as f64
}

impl HeraldryBarDots {
    pub(super) fn new_bar(col_a: &DirectColour, col_b: &DirectColour, prop: u32, len: u32, dir: bool) -> HeraldryBarDots {
        HeraldryBarDots { 
            col_a: col_a.clone(), col_b: col_b.clone(), 
            prop, 
            len: len.max(1),
            dir, variety: Variety::Bar
        }
    }

    pub(super) fn new_dots(col_a: &DirectColour, col_b: &DirectColour, prop: u32, len: u32, dir: bool) -> HeraldryBarDots {
        HeraldryBarDots {
            col_a: col_a.clone(), col_b: col_b.clone(), 
            prop, 
            len: len.max(1),
            dir, variety: Variety::Dots
        }
    }

    pub(super) fn handle_type(&self) -> HeraldryHandleType {
        match self.variety {
            Variety::Bar => HeraldryHandleType::Crisp,
            Variety::Dots => HeraldryHandleType::HorizVert
        }
    }

    pub(super) fn scale(&self) -> HeraldryScale {
        match self.variety {
            Variety::Bar => HeraldryScale::Squeeze,
            Variety::Dots => HeraldryScale::Overrun
        }
    }

    pub(super) fn padding(&self) -> (u32,u32) { (0,0) }

    pub(super) fn rotate(&self) -> HeraldryBarDots {
        let mut out = self.clone();
        out.dir = !self.dir;
        out
    }

    fn full_size(&self, bitmap_multiplier: f64) -> (u32,u32) {
        (((BAR_WIDTH as f64)*bitmap_multiplier).round() as u32,
         ((BAR_WIDTH as f64)*bitmap_multiplier).round() as u32)
    }

    pub(super) fn size(&self, bitmap_multiplier: f64) -> (u32,u32) {
        let unit = self.full_size(bitmap_multiplier);
        let mut out = (BAR_WIDTH*self.len,unit.1);
        if self.dir { out = (out.1,out.0); }
        out
    }

    fn unit(&self, bitmap_multiplier: f64, prop: f64) -> (u32,u32) {
        let len = (self.len as f64) * bitmap_multiplier;
        if self.dir { ((BAR_WIDTH as f64*bitmap_multiplier) as u32,(len*prop) as u32) } else { ((len*prop) as u32,((BAR_WIDTH as f64*bitmap_multiplier)) as u32) }
    }

    fn draw_one(&self, canvas: &CanvasAndContext, text_origin: (u32,u32), x: u32, y: u32) -> Result<(),Error> {
        let bitmap_multiplier = sanitise_mult(canvas.bitmap_multiplier().round());
        let prop = (self.prop as f64)/100.;
        let unit = self.unit(bitmap_multiplier,1.);
        let extent = self.unit(bitmap_multiplier,prop);
        let origin = (text_origin.0+x*unit.0,text_origin.1+y*unit.1);
        canvas.rectangle(origin,unit,&self.col_a,false)?;
        canvas.path(origin,&[
            (0,       0),
            (extent.0,0),
            (extent.0,extent.1),
            (0,       extent.1)
        ],&self.col_b)?;
        Ok(())
    }
    
    pub(super) fn draw(&self, canvas: &mut CanvasAndContext, text_origin: (u32,u32), size: (u32,u32)) -> Result<(),Error> {
        let bitmap_multiplier = sanitise_mult(canvas.bitmap_multiplier().round());
        let unit = self.unit(bitmap_multiplier,1.);
        let count = (size.0/unit.0,size.1/unit.1);
        for y in 0..count.1 {
            for x in 0..count.0 {
                self.draw_one(canvas,text_origin,x,y)?;
            }
        }
        Ok(())
    }
}
