use std::sync::{Arc, Mutex};
use std::{f64::consts::PI, fmt::Debug, hash::Hash };
use peregrine_toolkit::error::Error;
use peregrine_toolkit::{identitynumber, hashable, lock};
use peregrine_toolkit::plumbing::lease::Lease;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement };
use peregrine_data::{ DirectColour, PenGeometry, Background };
use crate::shape::core::bitmap::Bitmap;
use super::canvas::Canvas;
use super::{bindery::SelfManagedWebGlTexture, weave::CanvasWeave};

const MIN_ROUNDING_SIZE: u32 = 8; // px :should be configurable in Background object if anyone wants it
const MAX_ROUNDING_SIZE: u32 = 16; // px :should be configurable in Background object if anyone wants it

fn pen_to_font(pen: &PenGeometry, bitmap_multiplier: f64) -> String {
    format!("{}px {}",(pen.size_in_webgl() * bitmap_multiplier).round(),pen.name())
}

fn colour_to_css(c: &DirectColour) -> String {
    format!("rgb({},{},{},{})",c.0,c.1,c.2,c.3)
}

fn draw_png_onload(context: CanvasRenderingContext2d, el: &HtmlImageElement, origin: (u32,u32), size: (u32,u32)) -> Result<(),JsValue> {
    context.draw_image_with_html_image_element_and_dw_and_dh(el,origin.0 as f64,origin.1 as f64,size.0 as f64,size.1 as f64)?;
    Ok(())
}

fn sub(a: u32, b: u32) -> u32 { a.max(b) - b } // avoiding underflow

identitynumber!(IDS);
hashable!(CanvasAndContext,id);

pub(crate) struct CanvasAndContext {
    id: u64,
    bitmap_multiplier: f64,
    element: Option<Lease<Canvas>>,
    context: Option<CanvasRenderingContext2d>,
    weave: CanvasWeave,
    font: Option<String>,
    font_height: Option<u32>,
    discarded: bool,
    gl_texture: Option<SelfManagedWebGlTexture>,
    is_active: bool,
}

impl Debug for CanvasAndContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CanvasAndContext").field("id", &self.id).finish()
    }
}

impl CanvasAndContext {
    pub(super) fn new(el: Lease<Canvas>, weave: &CanvasWeave, size: (u32,u32), bitmap_multiplier: f32) -> Result<CanvasAndContext,Error> {
        let context = el.get().element()
            .get_context("2d").map_err(|_| Error::fatal("cannot get 2d context"))?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>().map_err(|_| Error::fatal("cannot get 2d context"))?;
        Ok(CanvasAndContext {
            id: IDS.next(),
            bitmap_multiplier: bitmap_multiplier as f64,
            element: Some(el),
            context: Some(context),
            weave: weave.clone(),
            font: None,
            font_height: None,
            discarded: false,
            gl_texture: None,
            is_active: false,
        })
    }

    pub(crate) fn id(&self) -> u64 { self.id }
    pub(crate) fn bitmap_multiplier(&self) -> f64 { self.bitmap_multiplier }
    pub(crate) fn get_gl_texture(&self) -> Option<&SelfManagedWebGlTexture> { self.gl_texture.as_ref() }
    pub(crate) fn get_gl_texture_mut(&mut self) -> Option<&mut SelfManagedWebGlTexture> { self.gl_texture.as_mut() }
    pub(crate) fn set_gl_texture(&mut self, texture: Option<SelfManagedWebGlTexture>) { self.gl_texture = texture; }
    pub(crate) fn is_active(&mut self) -> &mut bool { &mut self.is_active }

    pub(crate) fn set_font(&mut self, pen: &PenGeometry) -> Result<(),Error> {
        if self.discarded { return Err(Error::fatal("set_font on discarded flat canvas")); }
        let new_font = pen_to_font(pen,self.bitmap_multiplier);
        if let Some(old_font) = &self.font {
            if *old_font == new_font { return Ok(()); }
        }
        self.font = Some(new_font.to_string());
        self.font_height = Some((pen.size_in_webgl()*self.bitmap_multiplier) as u32);
        self.context()?.set_font(self.font.as_ref().unwrap());
        Ok(())
    }

    pub(crate) fn measure(&mut self, text: &str) -> Result<(u32,u32),Error> {
        if self.discarded { return Err(Error::fatal("set_font on discarded flat canvas")); }
        let width = self.context.as_ref().unwrap().measure_text(text).map_err(|e| Error::fatal(&format!("cannot measure text: {:?}",e)))?.width();
        let height = self.font_height.ok_or_else(|| Error::fatal("no font set before measure"))?;
        Ok((width as u32,height as u32))
    }

    pub(crate) fn rectangle(&self, origin: (u32,u32), size: (u32,u32), colour: &DirectColour, multiply: bool) -> Result<(),Error> {
        if self.discarded { return Err(Error::fatal("set_font on discarded flat canvas")); }
        let context = self.context()?;
        context.set_fill_style(&colour_to_css(colour).into()); // TODO background colours for pen
        let bitmap_multiplier = if multiply { self.bitmap_multiplier } else { 1. };
        context.fill_rect(origin.0 as f64 * bitmap_multiplier, origin.1 as f64 * bitmap_multiplier,
            size.0 as f64 * bitmap_multiplier, size.1 as f64 * bitmap_multiplier);
        Ok(())
    }

    pub(crate) fn circle(&self, origin: (u32,u32), radius: u32, colour: &DirectColour, multiply: bool) -> Result<(),Error> {
        if self.discarded { return Err(Error::fatal("circle on discarded flat canvas")); }
        let multiplier = if multiply { self.bitmap_multiplier } else { 1. };
        let origin = (origin.0 as f64 * multiplier, origin.1 as f64 * multiplier);
        let radius = radius as f64 * multiplier;
        let context = self.context()?;
        context.begin_path();
        context.arc(origin.0,origin.1,radius-1.,0.,2.*PI).map_err(|_x| Error::fatal("cannot draw arc"))?;
        context.set_fill_style(&colour_to_css(colour).into());
        context.fill();
        Ok(())
    }

    fn draw_png_real(&self, context: CanvasRenderingContext2d, name: Option<String>, origin: (u32,u32), size: (u32,u32), bitmap: &mut Bitmap) -> Result<(),JsValue> {
        bitmap.onload(move |el| {
            draw_png_onload(context,el,origin,size);
        });
        Ok(())
    }    

    pub(crate) fn draw_png(&self, name: Option<String>, origin: (u32,u32), size: (u32,u32), bitmap: &mut Bitmap) -> Result<(),Error> {
        if self.discarded { return Err(Error::fatal("set_font on discarded flat canvas")); }
        let context = self.context()?.clone();
        self.draw_png_real(context,name,origin,size,bitmap).map_err(|_| Error::fatal("cannot carate png"))?;
        Ok(())
    }

    pub(crate) fn path(&self, origin: (u32,u32), path: &[(u32,u32)], colour: &DirectColour) -> Result<(),Error> {
        if self.discarded { return Err(Error::fatal("set_font on discarded flat canvas")); }
        let context = self.context()?;
        context.set_fill_style(&colour_to_css(colour).into()); // TODO background colours for pen
        context.begin_path();
        let mut first = true;
        for point in path {
            let (x,y) = ((point.0+origin.0) as f64, (point.1+origin.1) as f64);
            if first {
                context.move_to(x,y);
            } else {
                context.line_to(x,y);
            }
            first = false;
        }
        context.close_path();
        context.fill();
        Ok(())
    }

    pub(crate) fn background(&self, origin: (u32,u32), size: (u32,u32), background: &Background, multiply: bool) -> Result<(),Error> {
        if background.round {
            let d = (size.0/2).min(size.1/2).min(MAX_ROUNDING_SIZE).max(MIN_ROUNDING_SIZE);
            let d = 16;
            self.rectangle((origin.0+d,origin.1),(sub(size.0,2*d+1),size.1),&background.colour,multiply)?;
            self.rectangle((origin.0,origin.1+d),(size.0,sub(size.1,2*d+1)),&background.colour,multiply)?;
            let nw = (origin.0 + d,origin.1 + d);
            let ne = (sub(nw.0 + size.0, 2*d+1),nw.1);
            let sw = (nw.0, sub(nw.1 + size.1,2*d+1));
            let se = (ne.0,sw.1);

            self.circle(nw,d,&background.colour,multiply)?;
            self.circle(ne,d,&background.colour,multiply)?;
            self.circle(sw,d,&background.colour,multiply)?;
            self.circle(se,d,&background.colour,multiply)?;
        } else {
            self.rectangle(origin,size,&background.colour,multiply)?;
        }
        Ok(())
    }

    pub(crate) fn text(&self, text: &str, origin: (u32,u32), colour: &DirectColour) -> Result<(),Error> {
        if self.discarded { return Err(Error::fatal("set_font on discarded flat canvas")); }
        let context = self.context()?;
        context.set_text_baseline("top");
        context.set_fill_style(&colour_to_css(&colour).into());
        context.fill_text(text,origin.0 as f64,origin.1 as f64).map_err(|e| Error::fatal(&format!("fill_text failed: {:?}",e)))?;
        Ok(())
    }

    pub(crate) fn size(&self) -> (u32,u32) { self.element.as_ref().unwrap().get().size() }
    pub(crate) fn weave(&self) -> &CanvasWeave { &self.weave }
    pub(crate) fn element(&self) -> Result<&HtmlCanvasElement,Error> {
        if self.discarded { return Err(Error::fatal("set_font on discarded flat canvas")); }
        Ok(&self.element.as_ref().unwrap().get().element())
    }

    pub(super) fn context(&self) -> Result<&CanvasRenderingContext2d,Error> {
        if self.discarded { return Err(Error::fatal("set_font on discarded flat canvas")); }
        Ok(&self.context.as_ref().unwrap())
    }
}

impl Drop for CanvasAndContext {
    fn drop(&mut self) {
        self.element = None;
        self.context = None;
        self.font = None;
        self.discarded = true;
    }
}

#[derive(Clone)]
pub struct CanvasInUse(Arc<Mutex<CanvasAndContext>>);

impl CanvasInUse {
    pub(crate) fn new(lease: Lease<Canvas>, weave: &CanvasWeave, size: (u32,u32), bitmap_multiplier: f32) -> Result<CanvasInUse,Error> {
        Ok(CanvasInUse(Arc::new(Mutex::new(CanvasAndContext::new(lease,weave,size,bitmap_multiplier)?))))
    }

    pub(crate) fn retrieve<F,X>(&self, cb: F) -> X
            where F: FnOnce(&CanvasAndContext) -> X {
        let y = lock!(self.0);
        cb(&y)
    }

    pub(crate) fn modify<F,X>(&self, cb: F) -> X
            where F: FnOnce(&mut CanvasAndContext) -> X {
        let mut y = lock!(self.0);
        cb(&mut y)
    }
}

impl PartialEq for CanvasInUse {
    fn eq(&self, other: &Self) -> bool {
        let a = lock!(self.0).id();
        let b = lock!(other.0).id();
        a == b
    }
}

impl Eq for CanvasInUse {}

impl Hash for CanvasInUse {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        lock!(self.0).id().hash(state);
    }
}

#[cfg(debug_assertions)]
impl Debug for CanvasInUse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        lock!(self.0).fmt(f)
    }
}
