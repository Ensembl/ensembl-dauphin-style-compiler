use std::collections::hash_map::DefaultHasher;
use std::hash::{ Hash, Hasher };
use std::sync::{Arc, Mutex};
use peregrine_data::{ DirectColour };
use peregrine_toolkit::error::Error;
use peregrine_toolkit::lock;
use crate::shape::core::flatdrawing::{FlatDrawingItem, FlatDrawingManager, CanvasItemHandle};
use crate::shape::core::texture::CanvasTextureArea;
use crate::shape::layers::drawingtools::ToolPreparations;
use crate::webgl::{CanvasAndContext, CanvasInUse};
use crate::webgl::global::WebGlGlobal;
use crate::util::message::Message;
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
        Heraldry::BarDots(HeraldryBarDots::new_dots(col_a,col_b,prop,number,dir))
    }

    pub(crate) fn new_bar(col_a: &DirectColour, col_b: &DirectColour, prop: u32, number: (u32,u32), dir: bool) -> Heraldry {
        Heraldry::BarDots(HeraldryBarDots::new_bar(col_a,col_b,prop,number,dir))
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
}

impl FlatDrawingItem for Heraldry {
    fn calc_size(&mut self, gl: &mut WebGlGlobal) -> Result<(u32,u32),Error> {
        let bitmap_multiplier = gl.refs().canvas_source.bitmap_multiplier();
        Ok(match self {
            Heraldry::Stripe(_,_,_,count) => (STAMP*count.0,STAMP*count.1),
            Heraldry::BarDots(dots) => dots.size(bitmap_multiplier as f64)
        })
    }

    fn padding(&mut self, _: &mut WebGlGlobal) -> Result<(u32,u32),Error> {
        Ok(match  self {
            Heraldry::Stripe(_,_,_,_) => (PAD,PAD),
            Heraldry::BarDots(bardots) => bardots.padding()
        })
    }

    fn compute_hash(&self) -> Option<u64> {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        Some(hasher.finish())
    }

    fn build(&mut self, canvas: &mut CanvasAndContext, text_origin: (u32,u32), size: (u32,u32)) -> Result<(),Error> {
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
#[cfg_attr(debug_assertions,derive(Debug))]
pub(crate) enum HeraldryHandle {
    HorizVert(CanvasItemHandle,CanvasItemHandle),
    Horiz(CanvasItemHandle),
    Crisp(CanvasItemHandle)
}

#[derive(Clone,PartialEq,Eq,Hash,Debug)]
pub(crate) enum HeraldryCanvas {
    Horiz,
    Vert,
    Crisp
}

pub struct DrawingHeraldry {
    horiz: FlatDrawingManager,
    vert: FlatDrawingManager,
    crisp: FlatDrawingManager
}

impl DrawingHeraldry {
    pub fn new() -> DrawingHeraldry { 
        DrawingHeraldry {
            horiz: FlatDrawingManager::new(),
            vert: FlatDrawingManager::new(),
            crisp: FlatDrawingManager::new()
        }
    }

    pub(crate) fn add(&mut self, heraldry: Heraldry) -> HeraldryHandle {
        match heraldry.handle_type() {
            HeraldryHandleType::Horiz => {
                HeraldryHandle::Horiz(self.horiz.add(heraldry))
            },
            HeraldryHandleType::Crisp => {
                HeraldryHandle::Crisp(self.crisp.add(heraldry))
            },
            HeraldryHandleType::HorizVert => {
                let heraldry_rotated = heraldry.rotate();
                HeraldryHandle::HorizVert(self.horiz.add(heraldry_rotated),self.vert.add(heraldry))        
            }
        }
    }

    pub(crate) async fn calculate_requirements(&mut self, gl: &Arc<Mutex<WebGlGlobal>>, preparations: &mut ToolPreparations) -> Result<(),Error> {
        let mut gl = lock!(gl);
        self.horiz.calculate_requirements(&mut gl,preparations.heraldry_h_manager())?;
        self.vert.calculate_requirements(&mut gl,preparations.heraldry_v_manager())?;
        self.crisp.calculate_requirements(&mut gl,preparations.crisp_manager())?;
        Ok(())
    }

    pub(crate) fn get_texture_area_on_bitmap(&self, handle: &HeraldryHandle, canvas: &HeraldryCanvas) -> Result<Option<CanvasTextureArea>,Message> {
        Ok(match (canvas,handle) {
            (HeraldryCanvas::Horiz,HeraldryHandle::Horiz(h)) => Some(self.horiz.get_texture_areas_on_bitmap(h)?),
            (HeraldryCanvas::Horiz,HeraldryHandle::HorizVert(h,_)) => Some(self.horiz.get_texture_areas_on_bitmap(h)?),
            (HeraldryCanvas::Vert,HeraldryHandle::HorizVert(_,v)) => Some(self.vert.get_texture_areas_on_bitmap(v)?),
            (HeraldryCanvas::Crisp,HeraldryHandle::Crisp(h)) => Some(self.crisp.get_texture_areas_on_bitmap(h)?),
            _ => None
        })
    }

    pub(crate) fn draw_at_locations(&mut self, preparations: &mut ToolPreparations) -> Result<(),Error> {
        self.horiz.draw_at_locations(preparations.heraldry_h_manager())?;
        self.vert.draw_at_locations(preparations.heraldry_v_manager())?;
        self.crisp.draw_at_locations(preparations.crisp_manager())?;
        Ok(())
    }

    pub(crate) fn canvas_id(&self, canvas: &HeraldryCanvas) -> Option<CanvasInUse> {
        match canvas {
            HeraldryCanvas::Horiz => self.horiz.canvas_id(),
            HeraldryCanvas::Vert => self.vert.canvas_id(),
            HeraldryCanvas::Crisp => self.crisp.canvas_id()
        }
    }
}
