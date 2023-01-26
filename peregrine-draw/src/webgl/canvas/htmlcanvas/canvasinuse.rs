use std::sync::{Arc, Mutex};
use std::{f64::consts::PI, fmt::Debug, hash::Hash };
use peregrine_toolkit::error::Error;
use peregrine_toolkit::{identitynumber, hashable, lock};
use peregrine_toolkit::plumbing::lease::Lease;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlImageElement, WebGlRenderingContext, HtmlCanvasElement };
use peregrine_data::{ DirectColour, PenGeometry };
use crate::shape::canvasitem::bitmap::Bitmap;
use crate::webgl::canvas::binding::texturebinding::{TextureBindingSlot, TextureBinding};
use crate::webgl::canvas::binding::weave::CanvasWeave;
use crate::webgl::util::handle_context_errors2;

const MIN_ROUNDING_SIZE: u32 = 8; // px :should be configurable in Background object if anyone wants it
const MAX_ROUNDING_SIZE: u32 = 16; // px :should be configurable in Background object if anyone wants it

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
    element: Lease<HtmlCanvasElement>,
    size: (u32,u32),
    context: Option<CanvasRenderingContext2d>,
    weave: CanvasWeave,
    font: Option<String>,
    font_height: Option<u32>,
    discarded: bool,
    texture: Option<TextureBindingSlot>
}

impl Debug for CanvasAndContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CanvasAndContext").field("id", &self.id).finish()
    }
}

impl CanvasAndContext {
    pub(super) fn new(el: Lease<HtmlCanvasElement>, size: (u32,u32), weave: &CanvasWeave, bitmap_multiplier: f32) -> Result<CanvasAndContext,Error> {
        let context = el.get()
            .get_context("2d").map_err(|_| Error::fatal("cannot get 2d context"))?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>().map_err(|_| Error::fatal("cannot get 2d context"))?;
        Ok(CanvasAndContext {
            id: IDS.next(),
            bitmap_multiplier: bitmap_multiplier as f64,
            element: el,
            size,
            context: Some(context),
            weave: weave.clone(),
            font: None,
            font_height: None,
            discarded: false,
            texture: None
        })
    }

    pub(crate) fn id(&self) -> u64 { self.id }
    pub(crate) fn bitmap_multiplier(&self) -> f64 { self.bitmap_multiplier }

    pub(crate) fn activate(&mut self, textures: &mut TextureBinding, context: &WebGlRenderingContext) -> Result<u32,Error> {
        if self.texture.is_none() {
            self.texture = Some(textures.new_token(context)?);
        }
        let (texture,slot) = self.texture.as_ref().unwrap().activate(&self.element.get(),&self.weave,context)?;
        context.active_texture(WebGlRenderingContext::TEXTURE0 + (slot as u32));
        handle_context_errors2(context)?;
        context.bind_texture(WebGlRenderingContext::TEXTURE_2D,Some(&texture));
        handle_context_errors2(context)?;
        Ok(slot)
    }

    pub(crate) fn set_font(&mut self, pen: &PenGeometry) -> Result<(),Error> {
        if self.discarded { return Err(Error::fatal("set_font on discarded flat canvas")); }
        let new_font = pen.to_font(self.bitmap_multiplier);
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

    fn draw_png_real(&self, context: CanvasRenderingContext2d, origin: (u32,u32), size: (u32,u32), bitmap: &Bitmap) -> Result<(),JsValue> {
        bitmap.onload(move |el| {
            draw_png_onload(context,el,origin,size);
        });
        Ok(())
    }    

    pub(crate) fn draw_png(&self, origin: (u32,u32), size: (u32,u32), bitmap: &Bitmap) -> Result<(),Error> {
        if self.discarded { return Err(Error::fatal("set_font on discarded flat canvas")); }
        let context = self.context()?.clone();
        self.draw_png_real(context,origin,size,bitmap).map_err(|_| Error::fatal("cannot carate png"))?;
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

    pub(crate) fn background(&self, origin: (u32,u32), size: (u32,u32), background: &DirectColour, multiply: bool) -> Result<(),Error> {
        self.rectangle(origin,size,background,multiply)?;
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

    pub(crate) fn size(&self) -> (u32,u32) { self.size }

    pub(super) fn context(&self) -> Result<&CanvasRenderingContext2d,Error> {
        if self.discarded { return Err(Error::fatal("set_font on discarded flat canvas")); }
        Ok(&self.context.as_ref().unwrap())
    }
}

impl Drop for CanvasAndContext {
    fn drop(&mut self) {
        self.context = None;
        self.font = None;
        self.discarded = true;
    }
}

#[derive(Clone)]
pub struct CanvasInUse(Arc<Mutex<CanvasAndContext>>);

impl CanvasInUse {
    pub(crate) fn new(lease: Lease<HtmlCanvasElement>, size: (u32,u32), weave: &CanvasWeave, bitmap_multiplier: f32) -> Result<CanvasInUse,Error> {
        Ok(CanvasInUse(Arc::new(Mutex::new(CanvasAndContext::new(lease,size,weave,bitmap_multiplier)?))))
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
