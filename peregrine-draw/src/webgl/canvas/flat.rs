use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlImageElement };
use peregrine_data::{ Pen, DirectColour };
use super::{bindery::SelfManagedWebGlTexture, canvasstore::HtmlFlatCanvas, pngcache::PngCache, weave::CanvasWeave};
use crate::util::{message::Message, fonts::Fonts};
use super::canvasstore::CanvasStore;
use peregrine_toolkit::{js::exception::js_result_to_option_console };

fn pen_to_font(pen: &Pen, bitmap_multiplier: f64) -> String {
    format!("{}px {}",(pen.size_in_webgl() * bitmap_multiplier).round(),pen.name())
}

fn colour_to_css(c: &DirectColour) -> String {
    format!("rgb({},{},{})",c.0,c.1,c.2)
}

fn draw_png_onload(context: CanvasRenderingContext2d, el: HtmlImageElement, origin: (u32,u32), size: (u32,u32)) -> Result<(),JsValue> {
    context.draw_image_with_html_image_element_and_dw_and_dh(&el,origin.0 as f64,origin.1 as f64,size.0 as f64,size.1 as f64)?;
    Ok(())
}

pub(crate) struct Flat {
    bitmap_multiplier: f64,
    element: Option<HtmlFlatCanvas>,
    context: Option<CanvasRenderingContext2d>,
    weave: CanvasWeave,
    font: Option<String>,
    font_height: Option<u32>,
    size: (u32,u32),
    discarded: bool,
    gl_texture: Option<SelfManagedWebGlTexture>,
    is_active: bool,
    png_cache: PngCache
}

impl Flat {
    pub(super) fn new(canvas_store: &mut CanvasStore, png_cache: &PngCache, document: &Document, weave: &CanvasWeave, size: (u32,u32), bitmap_multiplier: f32) -> Result<Flat,Message> {
        let el = canvas_store.allocate(document, size.0, size.1, weave.round_up())?;
        let context = el.element()
            .get_context("2d").map_err(|_| Message::Canvas2DFailure("cannot get 2d context".to_string()))?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>().map_err(|_| Message::Canvas2DFailure("cannot get 2d context".to_string()))?;
        Ok(Flat {
            bitmap_multiplier: bitmap_multiplier as f64,
            size: el.size(),
            element: Some(el),
            context: Some(context),
            weave: weave.clone(),
            font: None,
            font_height: None,
            discarded: false,
            gl_texture: None,
            is_active: false,
            png_cache: png_cache.clone()
        })
    }

    pub(crate) fn bitmap_multiplier(&self) -> f64 { self.bitmap_multiplier }
    pub(crate) fn get_gl_texture(&self) -> Option<&SelfManagedWebGlTexture> { self.gl_texture.as_ref() }
    pub(crate) fn set_gl_texture(&mut self, texture: Option<SelfManagedWebGlTexture>) { self.gl_texture = texture; }
    pub(crate) fn is_active(&mut self) -> &mut bool { &mut self.is_active }

    pub(crate) fn set_font(&mut self, pen: &Pen) -> Result<(),Message> {
        if self.discarded { return Err(Message::CodeInvariantFailed(format!("set_font on discarded flat canvas"))); }
        let new_font = pen_to_font(pen,self.bitmap_multiplier);
        if let Some(old_font) = &self.font {
            if *old_font == new_font { return Ok(()); }
        }
        self.font = Some(new_font.to_string());
        self.font_height = Some((pen.size_in_webgl()*self.bitmap_multiplier) as u32);
        self.context()?.set_font(self.font.as_ref().unwrap());
        Ok(())
    }

    pub(crate) fn measure(&mut self, text: &str) -> Result<(u32,u32),Message> {
        if self.discarded { return Err(Message::CodeInvariantFailed(format!("set_font on discarded flat canvas"))); }
        let width = self.context.as_ref().unwrap().measure_text(text).map_err(|e| Message::Canvas2DFailure(format!("cannot measure text: {:?}",e)))?.width();
        let height = self.font_height.ok_or_else(|| Message::CodeInvariantFailed("no font set before measure".to_string()))?;
//        log!("width of {:?} is {:?} (canvas is {:?} font {:?} fh {:?})",text,width,self.size.0,self.font,self.font_height);
        Ok((width as u32,height as u32))
    }

    pub(crate) fn rectangle(&self, origin: (u32,u32), size: (u32,u32), colour: &DirectColour, multiply: bool) -> Result<(),Message> {
        if self.discarded { return Err(Message::CodeInvariantFailed(format!("set_font on discarded flat canvas"))); }
        let context = self.context()?;
        context.set_fill_style(&colour_to_css(colour).into()); // TODO background colours for pen
        let bitmap_multiplier = if multiply { self.bitmap_multiplier } else { 1. };
        context.fill_rect(origin.0 as f64 * bitmap_multiplier, origin.1 as f64 * bitmap_multiplier,
            size.0 as f64 * bitmap_multiplier, size.1 as f64 * bitmap_multiplier);
        Ok(())
    }

    fn draw_png_real(&self, context: CanvasRenderingContext2d, name: Option<String>, origin: (u32,u32), size: (u32,u32), data: &[u8]) -> Result<(),JsValue> {
        if let Some(name) = &name {
            if let Some(el) = self.png_cache.get(name) {
                draw_png_onload(context,el,origin,size)?;
                return Ok(());
            }
        }
        let ascii_data = base64::encode(data);
        let img = HtmlImageElement::new()?;
        let img2 = img.clone();
        img.set_src(&format!("data:image/png;base64,{0}",ascii_data));
        let png_cache = self.png_cache.clone();
        let closure = Closure::once_into_js(move || {
            js_result_to_option_console(draw_png_onload(context,img2.clone(),origin,size));
            if let Some(name) = &name {
                png_cache.set(name,img2.clone());
            }
        });
        img.set_onload(Some(&closure.as_ref().unchecked_ref()));
        Ok(())
    }    

    pub(crate) fn draw_png(&self,  name: Option<String>,origin: (u32,u32), size: (u32,u32), data: &[u8]) -> Result<(),Message> {
        if self.discarded { return Err(Message::CodeInvariantFailed(format!("set_font on discarded flat canvas"))); }
        let context = self.context()?.clone();
        self.draw_png_real(context,name,origin,size,data).map_err(|_| Message::Canvas2DFailure("cannot carate png".to_string()))?;
        Ok(())
    }

    pub(crate) fn clear(&self, origin: (u32,u32), size: (u32,u32)) -> Result<(),Message> {
        if self.discarded { return Err(Message::CodeInvariantFailed(format!("set_font on discarded flat canvas"))); }
        let context = self.context()?;
        context.clear_rect(origin.0 as f64, origin.1 as f64, size.0 as f64, size.1 as f64);
        Ok(())
    }

    pub(crate) fn path(&self, origin: (u32,u32), path: &[(u32,u32)], colour: &DirectColour) -> Result<(),Message> {
        if self.discarded { return Err(Message::CodeInvariantFailed(format!("set_font on discarded flat canvas"))); }
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

    // TODO white-bgd canvas
    pub(crate) fn text(&self, text: &str, origin: (u32,u32), size: (u32,u32), colour: &DirectColour, background: &DirectColour) -> Result<(),Message> {
        if self.discarded { return Err(Message::CodeInvariantFailed(format!("set_font on discarded flat canvas"))); }
        self.rectangle(origin,size,background,false)?;
        let context = self.context()?;
        context.set_text_baseline("top");
        context.set_fill_style(&colour_to_css(&colour).into());
        context.fill_text(text,origin.0 as f64,origin.1 as f64).map_err(|e| Message::Canvas2DFailure(format!("fill_text failed: {:?}",e)))?;
        Ok(())
    }

    pub(crate) fn size(&self) -> &(u32,u32) { &self.size }
    pub(crate) fn weave(&self) -> &CanvasWeave { &self.weave }
    pub(crate) fn element(&self) -> Result<&HtmlCanvasElement,Message> {
        if self.discarded { return Err(Message::CodeInvariantFailed(format!("set_font on discarded flat canvas"))); }
        Ok(&self.element.as_ref().unwrap().element())
    }

    pub(super) fn context(&self) -> Result<&CanvasRenderingContext2d,Message> {
        if self.discarded { return Err(Message::CodeInvariantFailed(format!("set_font on discarded flat canvas"))); }
        Ok(&self.context.as_ref().unwrap())
    }

    pub(super) fn discard(&mut self, canvas_store: &mut CanvasStore) -> Result<(),Message> {
        if self.discarded { return Ok(()); }
        canvas_store.free(self.element.take().unwrap());
        self.element = None;
        self.context = None;
        self.font = None;
        self.discarded = true;
        Ok(())
    }
}
