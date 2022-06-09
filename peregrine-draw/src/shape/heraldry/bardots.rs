use peregrine_data::DirectColour;
use crate::{Message, webgl::{Flat}};
use super::heraldry::{HeraldryHandleType, HeraldryScale};

/* A bar indicates a certain number of stripes and strecthes.
 * Dots represent a certain dot length and repeat.
 */

const BAR_WIDTH : u32 = 32;
const DOTS_REPEAT : u32 = 4;
const PAD : u32 = 8;

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Hash,Clone)]
enum Variety {
    Bar,
    Dots
}

fn round_down_pow_two(mut num: f64) -> u32 {
    let mut out = 1;
    while num >= 2. {
        out *= 2;
        num /= 2.;
    }
    out
}

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Hash,Clone)]
pub(crate) struct HeraldryBarDots {
    col_a: DirectColour,
    col_b: DirectColour,
    prop: u32,
    number: (u32,u32),
    dir: bool,
    variety: Variety
}

impl HeraldryBarDots {
    pub(super) fn new_bar(col_a: &DirectColour, col_b: &DirectColour, prop: u32, number: (u32,u32), dir: bool) -> HeraldryBarDots {
        HeraldryBarDots { col_a: col_a.clone(), col_b: col_b.clone(), prop, number, dir, variety: Variety::Bar }
    }

    pub(super) fn new_dots(col_a: &DirectColour, col_b: &DirectColour, prop: u32, number: (u32,u32), dir: bool) -> HeraldryBarDots {
        HeraldryBarDots { col_a: col_a.clone(), col_b: col_b.clone(), prop, number, dir, variety: Variety::Dots }
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

    pub(super) fn padding(&mut self) -> (u32,u32) {
        match self.variety {
            Variety::Bar => (PAD,PAD),
            Variety::Dots => (0,0),
        }
    }

    pub(super) fn rotate(&self) -> HeraldryBarDots {
        let mut out = self.clone();
        out.number = (out.number.1,out.number.0);
        out.dir = !self.dir;
        out
    }

    fn unit_size(&self, bitmap_multiplier: f64) -> (u32,u32) {
        let rounded_bitmap_multiplier = round_down_pow_two(bitmap_multiplier);
        match self.variety {
            Variety::Bar =>   (BAR_WIDTH,BAR_WIDTH),
            Variety::Dots =>  if !self.dir {
                (self.number.0*rounded_bitmap_multiplier,self.number.1*rounded_bitmap_multiplier)
            } else {
                (self.number.1*rounded_bitmap_multiplier,self.number.0*rounded_bitmap_multiplier)
            }
        }
    }

    pub(super) fn size(&self, bitmap_multiplier: f64) -> (u32,u32) {
        let unit = self.unit_size(bitmap_multiplier);
        let mut out = match self.variety {
            Variety::Bar => (unit.0*self.number.0,unit.1),
            Variety::Dots => (self.number.0*DOTS_REPEAT*2,self.number.1*2)
        };
        if self.dir { out = (out.1,out.0); }
        out
    }

    fn draw_one(&self, canvas: &Flat, text_origin: (u32,u32), x: u32, y: u32) -> Result<(),Message> {
        let bitmap_multiplier = canvas.bitmap_multiplier();
        let unit = self.unit_size(bitmap_multiplier);
        let t = (text_origin.0+x*unit.0,text_origin.1+y*unit.1);
        let extent= if self.dir { (100,self.prop) } else { (self.prop,100) };
        let offset= if self.dir { (0,50-self.prop/2) } else { (50-self.prop/2,0) };
        let extent = ((extent.0*unit.0) / 100,(extent.1*unit.1) / 100);
        let offset = ((offset.0*unit.0) / 100,(offset.1*unit.1) / 100);
        canvas.rectangle(t,unit,&self.col_a,true)?;
        canvas.path((t.0+offset.0,t.1+offset.1),&[
            (0,       0),
            (extent.0,0),
            (extent.0,extent.1),
            (0,       extent.1)
        ],&self.col_b)?;
        Ok(())
    }
    
    pub(super) fn draw(&self, canvas: &mut Flat, text_origin: (u32,u32), size: (u32,u32)) -> Result<(),Message> {
        let bitmap_multiplier = canvas.bitmap_multiplier();
        let unit = self.unit_size(bitmap_multiplier);
        let count = (size.0/unit.0,size.1/unit.1);
        for y in 0..count.1 {
            for x in 0..count.0 {
                self.draw_one(canvas,text_origin,x,y)?;
            }
        }
        Ok(())
    }
}
