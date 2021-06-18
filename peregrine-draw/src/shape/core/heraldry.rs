use std::collections::hash_map::DefaultHasher;
use std::hash::{ Hash, Hasher };
use std::iter::Inspect;
use peregrine_data::{ DirectColour };
use keyed::keyed_handle;
use crate::shape::layers::drawing::ToolPreparations;
use crate::webgl::canvas::flatplotallocator::FlatPositionManager;
use crate::webgl::{Flat, FlatStore};
use crate::webgl::global::WebGlGlobal;
use super::flatdrawing::{FlatDrawingItem, FlatDrawingManager};
use super::texture::CanvasTextureArea;
use crate::util::message::Message;
use crate::webgl::canvas::flatstore::FlatId;

keyed_handle!(InternalHeraldryHandle);

const STAMP : u32 = 32;
const PAD : u32 = 8;

fn pad(z: (u32,u32)) -> (u32,u32) {
    (z.0+PAD,z.1+PAD)
}

fn stripe_stamp(canvas: &Flat, t: (u32,u32), m: (u32,u32), a: &DirectColour, b: &DirectColour, p: u32) -> Result<(),Message> {
    canvas.rectangle(m,(STAMP,STAMP),&DirectColour(0,0,0))?;
    canvas.rectangle(t,(STAMP,STAMP),b)?;
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

fn bar_stamp(canvas: &Flat, t: (u32,u32), m: (u32,u32), a: &DirectColour, b: &DirectColour, p: u32,horiz: bool) -> Result<(),Message> {
    let p = 100-p;
    let extent= if horiz { (100,p) } else { (p,100) };
    let offset= if horiz { (0,50-p/2) } else { (50-p/2,0) };
    let extent = ((extent.0*STAMP) / 100,(extent.1*STAMP) / 100);
    let offset = ((offset.0*STAMP) / 100,(offset.1*STAMP) / 100);
    canvas.rectangle(m,(STAMP,STAMP),&DirectColour(0,0,0))?;
    canvas.rectangle(t,(STAMP,STAMP),a)?;
    canvas.path((t.0+offset.0,t.1+offset.1),&[
        (0,       0),
        (extent.0,0),
        (extent.0,extent.1),
        (0,       extent.1)
    ],b)?;
    Ok(())
}

#[derive(Hash)]
pub(crate) enum Heraldry {
    Stripe(DirectColour,DirectColour,u32,(u32,u32)),
    Bar(DirectColour,DirectColour,u32,(u32,u32),bool),
    Dots(DirectColour,DirectColour,u32,(u32,u32),bool),
}

impl Heraldry {
    pub(crate) fn rotate(&self) -> Heraldry {
        match self {
            Heraldry::Stripe(a,b,p,(x,y)) => Heraldry::Stripe(a.clone(),b.clone(),*p,(*y,*x)),
            Heraldry::Bar(a,b,p,(x,y),dir) => Heraldry::Bar(a.clone(),b.clone(),*p,(*y,*x),!dir),
            Heraldry::Dots(a,b,p,(x,y),dir) => Heraldry::Bar(a.clone(),b.clone(),*p,(*y,*x),!dir),
        }
    }

    pub(crate) fn hollow(&self) -> bool {
        match self {
            Heraldry::Dots(_,_,_,_,_) => true,
            _ => false
        }
    }

    fn handle_type(&self) -> HeraldryHandleType {
        match self {
            Heraldry::Stripe(_,_,_,_) => HeraldryHandleType::Horiz,
            Heraldry::Bar(_,_,_,_,_) => HeraldryHandleType::Horiz,
            Heraldry::Dots(_,_,_,_,_) => HeraldryHandleType::HorizVert
        }
    }
}

impl FlatDrawingItem for Heraldry {
    fn calc_size(&mut self, _gl: &mut WebGlGlobal) -> Result<(u32,u32),Message> {
        Ok(match self {
            Heraldry::Stripe(_,_,_,count) => (STAMP*count.0,STAMP*count.1),
            Heraldry::Bar(_,_,_,count,false) => (STAMP*count.0,STAMP),
            Heraldry::Bar(_,_,_,count,true) => (STAMP,count.0*STAMP),
            Heraldry::Dots(_,_,_,count,false) => (STAMP*count.0,STAMP),
            Heraldry::Dots(_,_,_,count,true) => (STAMP,count.0*STAMP),
        })
    }

    fn padding(&mut self, _: &mut WebGlGlobal) -> Result<(u32,u32),Message> {
        Ok(match  self {
            Heraldry::Stripe(_,_,_,_) => (PAD,PAD),
            Heraldry::Bar(_,_,_,_,_) => (PAD,PAD),
            Heraldry::Dots(_,_,_,_,_) => (PAD,PAD),
        })
    }

    fn compute_hash(&self) -> Option<u64> {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        Some(hasher.finish())
    }

    fn build(&mut self, canvas: &mut Flat, text_origin: (u32,u32), mask_origin: (u32,u32), _size: (u32,u32)) -> Result<(),Message> {
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
            Heraldry::Bar(a,b,prop,count,horiz) |
            Heraldry::Dots(a,b,prop,count,horiz) => {
                for c in 0..count.0 {
                    let (x,y) = if *horiz { (0,c) } else { (c,0) };
                    let t = (text_origin.0+x*STAMP,text_origin.1+y*STAMP);
                    let m = (mask_origin.0+x*STAMP,mask_origin.1+y*STAMP);
                    bar_stamp(canvas,pad(t),pad(m),a,b,*prop,*horiz)?;
                }
            },
        }
        Ok(())
    }
}

enum HeraldryHandleType {
    HorizVert,
    Horiz
}

#[derive(Clone)]
pub(crate) enum HeraldryHandle {
    HorizVert(InternalHeraldryHandle,InternalHeraldryHandle),
    Horiz(InternalHeraldryHandle)
}

pub(crate) enum HeraldryCanvas {
    Horiz,
    Vert
}

pub struct DrawingHeraldry {
    horiz: FlatDrawingManager<InternalHeraldryHandle,Heraldry>,
    vert: FlatDrawingManager<InternalHeraldryHandle,Heraldry>
}

impl DrawingHeraldry {
    pub fn new() -> DrawingHeraldry { 
        DrawingHeraldry {
            horiz: FlatDrawingManager::new(),
            vert: FlatDrawingManager::new()
        }
    }

    pub(crate) fn add(&mut self, heraldry: Heraldry) -> HeraldryHandle {
        match heraldry.handle_type() {
            HeraldryHandleType::Horiz => {
                HeraldryHandle::Horiz(self.horiz.add(heraldry))
            },
            HeraldryHandleType::HorizVert => {
                let heraldry_rotated = heraldry.rotate();
                HeraldryHandle::HorizVert(self.horiz.add(heraldry),self.vert.add(heraldry_rotated))        
            }
        }
    }

    pub(crate) fn calculate_requirements(&mut self, gl: &mut WebGlGlobal, preparations: &mut ToolPreparations) -> Result<(),Message> {
        self.horiz.calculate_requirements(gl,preparations.heraldry_h_manager(),|_| {})?;
        self.vert.calculate_requirements(gl,preparations.heraldry_v_manager(),|_| {})?;
        Ok(())
    }

    pub(crate) fn get_texture_area(&self, handle: &HeraldryHandle, canvas: &HeraldryCanvas) -> Result<Option<CanvasTextureArea>,Message> {
        Ok(match (canvas,handle) {
            (HeraldryCanvas::Horiz,HeraldryHandle::Horiz(h)) => Some(self.horiz.get_texture_areas(h)?),
            (HeraldryCanvas::Horiz,HeraldryHandle::HorizVert(h,_)) => Some(self.horiz.get_texture_areas(h)?),
            (HeraldryCanvas::Vert,HeraldryHandle::HorizVert(_,v)) => Some(self.vert.get_texture_areas(v)?),
            _ => None
        })
    }

    pub(crate) fn draw_at_locations(&mut self, store: &mut FlatStore, preparations: &mut ToolPreparations) -> Result<(),Message> {
        self.horiz.draw_at_locations(store,preparations.heraldry_h_manager())?;
        self.vert.draw_at_locations(store,preparations.heraldry_v_manager())?;
        Ok(())
    }

    pub(crate) fn canvas_id(&self, canvas: &HeraldryCanvas) -> Option<FlatId> {
        match canvas {
            HeraldryCanvas::Horiz => self.horiz.canvas_id(),
            HeraldryCanvas::Vert => self.vert.canvas_id()
        }
    }
}
