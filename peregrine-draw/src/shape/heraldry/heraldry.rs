use std::collections::hash_map::DefaultHasher;
use std::hash::{ Hash, Hasher };
use peregrine_data::{ DirectColour };
use keyed::keyed_handle;
use crate::shape::core::flatdrawing::{FlatDrawingItem, FlatDrawingManager};
use crate::shape::core::texture::CanvasTextureArea;
use crate::shape::layers::drawing::ToolPreparations;
use crate::webgl::{Flat, FlatStore};
use crate::webgl::global::WebGlGlobal;
use crate::util::message::Message;
use crate::webgl::canvas::flatstore::FlatId;
use super::bardots::HeraldryBarDots;

keyed_handle!(InternalHeraldryHandle);

const STAMP : u32 = 32;
const PAD : u32 = 8;

fn pad(z: (u32,u32)) -> (u32,u32) {
    (z.0+PAD,z.1+PAD)
}

fn stripe_stamp(canvas: &Flat, t: (u32,u32), m: (u32,u32), a: &DirectColour, b: &DirectColour, p: u32) -> Result<(),Message> {
    canvas.rectangle(m,(STAMP,STAMP),&DirectColour(0,0,0,255),true)?;
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
    fn calc_size(&mut self, gl: &mut WebGlGlobal) -> Result<(u32,u32),Message> {
        let bitmap_multiplier = gl.refs().flat_store.bitmap_multiplier();
        Ok(match self {
            Heraldry::Stripe(_,_,_,count) => (STAMP*count.0,STAMP*count.1),
            Heraldry::BarDots(dots) => dots.size(bitmap_multiplier as f64)
        })
    }

    fn padding(&mut self, _: &mut WebGlGlobal) -> Result<(u32,u32),Message> {
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

    fn build(&mut self, canvas: &mut Flat, text_origin: (u32,u32), mask_origin: (u32,u32), size: (u32,u32)) -> Result<(),Message> {
        match self {
            Heraldry::Stripe(a,b,prop,count) => {
                let p = STAMP * (*prop) / 100;
                for y in 0..count.1 {
                    for x in 0..count.0 {
                        let t = (text_origin.0+x*STAMP,text_origin.1+y*STAMP);
                        let m = (mask_origin.0+x*STAMP,mask_origin.1+y*STAMP);
                        stripe_stamp(canvas,pad(t),pad(m),a,b,p)?;
                    }
                }
            },
            Heraldry::BarDots(dots) => {
                dots.draw(canvas,text_origin,mask_origin,size)?;
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
    HorizVert(InternalHeraldryHandle,InternalHeraldryHandle),
    Horiz(InternalHeraldryHandle),
    Crisp(InternalHeraldryHandle)
}

#[derive(Clone,PartialEq,Eq,Hash,Debug)]
pub(crate) enum HeraldryCanvas {
    Horiz,
    Vert,
    Crisp
}

pub struct DrawingHeraldry {
    horiz: FlatDrawingManager<InternalHeraldryHandle,Heraldry>,
    vert: FlatDrawingManager<InternalHeraldryHandle,Heraldry>,
    crisp: FlatDrawingManager<InternalHeraldryHandle,Heraldry>
}

impl DrawingHeraldry {
    pub fn new(bitmap_multiplier: f32) -> DrawingHeraldry { 
        DrawingHeraldry {
            horiz: FlatDrawingManager::new(bitmap_multiplier),
            vert: FlatDrawingManager::new(bitmap_multiplier),
            crisp: FlatDrawingManager::new(bitmap_multiplier)
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

    pub(crate) fn calculate_requirements(&mut self, gl: &mut WebGlGlobal, preparations: &mut ToolPreparations) -> Result<(),Message> {
        self.horiz.calculate_requirements(gl,preparations.heraldry_h_manager())?;
        self.vert.calculate_requirements(gl,preparations.heraldry_v_manager())?;
        self.crisp.calculate_requirements(gl,preparations.crisp_manager())?;
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

    pub(crate) fn draw_at_locations(&mut self, store: &mut FlatStore, preparations: &mut ToolPreparations) -> Result<(),Message> {
        self.horiz.draw_at_locations(store,preparations.heraldry_h_manager())?;
        self.vert.draw_at_locations(store,preparations.heraldry_v_manager())?;
        self.crisp.draw_at_locations(store,preparations.crisp_manager())?;
        Ok(())
    }

    pub(crate) fn canvas_id(&self, canvas: &HeraldryCanvas) -> Option<FlatId> {
        match canvas {
            HeraldryCanvas::Horiz => self.horiz.canvas_id(),
            HeraldryCanvas::Vert => self.vert.canvas_id(),
            HeraldryCanvas::Crisp => self.crisp.canvas_id()
        }
    }
}
