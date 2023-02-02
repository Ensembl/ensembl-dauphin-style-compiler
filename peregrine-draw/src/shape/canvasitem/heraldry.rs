use std::collections::hash_map::DefaultHasher;
use std::hash::{ Hash, Hasher };
use peregrine_data::{ DirectColour };
use peregrine_toolkit::error::Error;
use crate::shape::layers::drawingtools::{CanvasType, DrawingToolsBuilder};
use crate::shape::layers::patina::Freedom;
use crate::webgl::canvas::composition::canvasitem::{CanvasItemSize, CanvasItemAreaSource, CanvasItem};
use crate::webgl::canvas::htmlcanvas::canvasinuse::CanvasAndContext;
use crate::webgl::global::WebGlGlobal;
use super::bardots::HeraldryBarDots;

const STAMP : u32 = 32;
const PAD : u32 = 8;

fn pad(z: (u32,u32)) -> (u32,u32) {
    (z.0+PAD,z.1+PAD)
}

fn stripe_stamp(canvas: &CanvasAndContext, t: (u32,u32), a: &DirectColour, b: &DirectColour, p: u32) -> Result<(),Error> {
    canvas.rectangle(t,(STAMP,STAMP),b,true)?;
    canvas.path(t,&[
        (0,    0),
        (p,    0),
        (STAMP,STAMP-p),
        (STAMP,STAMP)
    ],a)?;
    canvas.path(t,&[
        (0,STAMP-p),
        (p,STAMP),
        (0,STAMP)
    ],a)?;
    Ok(())
}

#[derive(Clone,PartialEq,Eq,Hash,Debug)]
pub(crate) enum HeraldryScale {
    Squeeze,
    Overrun
}

impl HeraldryScale {
    pub fn is_free(&self) -> bool {
        match self {
            HeraldryScale::Overrun => true,
            HeraldryScale::Squeeze => false
        }
    }
}

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Hash,Clone)]
pub(crate) enum Heraldry {
    Stripe(DirectColour,DirectColour,u32,(u32,u32)),
    BarDots(HeraldryBarDots),
}

impl Heraldry {
    pub(crate) fn new_dots(col_a: &DirectColour, col_b: &DirectColour, prop: u32, number: (u32,u32), dir: bool) -> Heraldry {
        Heraldry::BarDots(HeraldryBarDots::new_dots(col_a,col_b,prop,number.0,dir))
    }

    pub(crate) fn new_bar(col_a: &DirectColour, col_b: &DirectColour, prop: u32, number: (u32,u32), dir: bool) -> Heraldry {
        Heraldry::BarDots(HeraldryBarDots::new_bar(col_a,col_b,prop,number.0,dir))
    }

    pub(crate) fn rotate(&self) -> Heraldry {
        match self {
            Heraldry::Stripe(a,b,p,(x,y)) => Heraldry::Stripe(a.clone(),b.clone(),*p,(*y,*x)),
            Heraldry::BarDots(dots) => Heraldry::BarDots(dots.rotate()),
        }
    }

    fn handle_type(&self) -> HeraldryHandleType {
        match self {
            Heraldry::Stripe(_,_,_,_) => HeraldryHandleType::Crisp,
            Heraldry::BarDots(bardots) => bardots.handle_type(),
        }
    }

    pub(crate) fn scale(&self) -> HeraldryScale {
        match self {
            Heraldry::Stripe(_,_,_,_) => HeraldryScale::Squeeze,
            Heraldry::BarDots(bardots) => bardots.scale(),
        }
    }

    pub(crate) fn canvases_used(&self) -> HeraldryCanvasesUsed {
        self.handle_type().canvases_used()
    }

    pub(crate) fn add(&self, manager: &mut DrawingToolsBuilder) -> Result<HeraldryHandle,Error> {
        Ok(match self.handle_type() {
            HeraldryHandleType::Horiz => {
                HeraldryHandle::Horiz(manager.composition_builder(&CanvasType::HeraldryHoriz).add(self.clone())?)
            },
            HeraldryHandleType::Crisp => {
                HeraldryHandle::Crisp(manager.composition_builder(&CanvasType::HeraldryVert).add(self.clone())?)
            },
            HeraldryHandleType::HorizVert => {
                let rotated = self.rotate(); // rotated is vertical line
                HeraldryHandle::HorizVert(
                    manager.composition_builder(&CanvasType::HeraldryHoriz).add(self.clone())?, // gets horiz line
                    manager.composition_builder(&CanvasType::HeraldryVert).add(rotated)? // gets vertical line
                )
            }
        })
    }
}

impl CanvasItem for Heraldry {
    fn calc_size(&self, gl: &mut WebGlGlobal) -> Result<CanvasItemSize,Error> {
        let bitmap_multiplier = gl.refs().canvas_source.bitmap_multiplier();
        let size = match self {
            Heraldry::Stripe(_,_,_,count) => (STAMP*count.0,STAMP*count.1),
            Heraldry::BarDots(dots) => dots.size(bitmap_multiplier as f64)
        };
        let padding = match self {
            Heraldry::Stripe(_,_,_,_) => (PAD,PAD),
            Heraldry::BarDots(bardots) => bardots.padding()
        };
        Ok(CanvasItemSize::new(size,padding))
    }

    fn compute_hash(&self) -> Option<u64> {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        Some(hasher.finish())
    }

    fn draw_on_bitmap(&self, canvas: &mut CanvasAndContext, text_origin: (u32,u32), size: (u32,u32)) -> Result<(),Error> {
        match self {
            Heraldry::Stripe(a,b,prop,count) => {
                let p = STAMP * (*prop) / 100;
                for y in 0..count.1 {
                    for x in 0..count.0 {
                        let t = (text_origin.0+x*STAMP,text_origin.1+y*STAMP);
                        stripe_stamp(canvas,pad(t),a,b,p)?;
                    }
                }
            },
            Heraldry::BarDots(dots) => {
                dots.draw(canvas,text_origin,size)?;
            },
        }
        Ok(())
    }
}

#[derive(Clone,PartialEq,Eq,Hash,Debug)]
pub(crate) enum HeraldryCanvasesUsed {
    Solid(HeraldryCanvas),
    Hollow(HeraldryCanvas,HeraldryCanvas)
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub(super) enum HeraldryHandleType {
    HorizVert,
    Horiz,
    Crisp
}

impl HeraldryHandleType {
    fn canvases_used(&self) -> HeraldryCanvasesUsed {
        match self {
            HeraldryHandleType::Crisp => HeraldryCanvasesUsed::Solid(HeraldryCanvas::Crisp),
            HeraldryHandleType::Horiz => HeraldryCanvasesUsed::Solid(HeraldryCanvas::Horiz),
            HeraldryHandleType::HorizVert => HeraldryCanvasesUsed::Hollow(HeraldryCanvas::Vert,HeraldryCanvas::Horiz)
        }
    }
}

#[derive(Clone)]
pub(crate) enum HeraldryHandle {
    HorizVert(CanvasItemAreaSource,CanvasItemAreaSource),
    Horiz(CanvasItemAreaSource),
    Crisp(CanvasItemAreaSource)
}

impl HeraldryHandle {
    pub(crate) fn get_texture_area_on_bitmap(&self, canvas: &HeraldryCanvas) -> Option<&CanvasItemAreaSource> {
        match (canvas,self) {
            (HeraldryCanvas::Horiz,HeraldryHandle::Horiz(h)) => Some(h),
            (HeraldryCanvas::Horiz,HeraldryHandle::HorizVert(h,_)) => Some(h),
            (HeraldryCanvas::Vert,HeraldryHandle::HorizVert(_,v)) => Some(v),
            (HeraldryCanvas::Crisp,HeraldryHandle::Crisp(c)) => Some(c),
            _ => None
        }
    }
}

#[derive(Clone,PartialEq,Eq,Hash,Debug)]
pub(crate) enum HeraldryCanvas {
    Horiz,
    Vert,
    Crisp
}

impl HeraldryCanvas {
    pub(crate) fn to_canvas_type(&self) -> CanvasType {
        match self {
            HeraldryCanvas::Horiz => CanvasType::HeraldryHoriz,
            HeraldryCanvas::Vert => CanvasType::HeraldryVert,
            HeraldryCanvas::Crisp => CanvasType::Crisp
        }
    }

    pub fn to_freedom(&self) -> Freedom {
        match self {
            HeraldryCanvas::Horiz => Freedom::Vertical,
            HeraldryCanvas::Vert => Freedom::Horizontal,
            HeraldryCanvas::Crisp => Freedom::None,
        }
    }
}
