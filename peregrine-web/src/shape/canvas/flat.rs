use anyhow::{ Context, anyhow as err, bail };
use crate::util::error::js_error;
use wasm_bindgen::JsCast;
use web_sys::{ Document, HtmlCanvasElement, CanvasRenderingContext2d };
use peregrine_core::{ Pen, DirectColour };
use super::weave::CanvasWeave;

fn pen_to_font(pen: &Pen) -> String {
    format!("{} {}px",pen.0,pen.1)
}

fn colour_to_css(c: &DirectColour) -> String {
    format!("rgb({},{},{})",c.0,c.1,c.2)
}

pub(crate) struct CanvasElement {
    element: Option<HtmlCanvasElement>,
    context: Option<CanvasRenderingContext2d>,
    weave: CanvasWeave,
    font: Option<String>,
    font_height: Option<u32>,
    size: (u32,u32),
    discarded: bool
}

impl CanvasElement {
    pub(super) fn new(document: &Document, weave: &CanvasWeave, size: (u32,u32)) -> anyhow::Result<CanvasElement> {
        let el = js_error(document.create_element("canvas")).context("creating canvas")?;
        let canvas_el = el.dyn_into::<HtmlCanvasElement>().map_err(|e| err!("could not cast canvas to HtmlCanvasElement"))?;
        canvas_el.set_width(size.0);
        canvas_el.set_height(size.1);
        let context = canvas_el
            .get_context("2d").map_err(|_| err!("cannot get 2d context"))?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>().map_err(|_| err!("cannot get 2d context"))?;
        Ok(CanvasElement {
            element: Some(canvas_el),
            context: Some(context),
            weave: weave.clone(),
            size,
            font: None,
            font_height: None,
            discarded: false
        })
    }

    pub(crate) fn set_font(&mut self, pen: &Pen) -> anyhow::Result<()> {
        if self.discarded { bail!("discarded flat canvas") }
        let new_font = pen_to_font(pen);
        if let Some(old_font) = &self.font {
            if *old_font == new_font { return Ok(()); }
        }
        self.font = Some(new_font.to_string());
        self.font_height = Some(pen.1 as u32);
        self.context()?.set_font(self.font.as_ref().unwrap());
        Ok(())
    }

    pub(crate) fn measure(&mut self, text: &str) -> anyhow::Result<(u32,u32)> {
        if self.discarded { bail!("discarded flat canvas") }
        let width = js_error(self.context.as_ref().unwrap().measure_text(text)).context("measuring text")?.width();
        let height = self.font_height.ok_or_else(|| err!("no font set before measure"))?;
        Ok((width as u32,height as u32))
    }

    // TODO white-bgd canvas
    pub(crate) fn text(&self, text: &str, origin: (u32,u32), size: (u32,u32), colour: &DirectColour) -> anyhow::Result<()> {
        if self.discarded { bail!("discarded flat canvas") }
        let context = self.context()?;
        context.set_fill_style(&colour_to_css(&DirectColour(255,255,255)).into()); // TODO background colours for pen
        context.fill_rect(origin.0 as f64, origin.1 as f64, size.0 as f64, size.1 as f64);
        context.set_text_baseline("top");
        context.set_fill_style(&colour_to_css(&colour).into());
        js_error(context.fill_text(text,origin.0 as f64,origin.1 as f64)).context("drawing text")?;
        Ok(())
    }

    pub(super) fn size(&self) -> &(u32,u32) { &self.size }
    pub(crate) fn weave(&self) -> &CanvasWeave { &self.weave }
    pub(crate) fn element(&self) -> anyhow::Result<&HtmlCanvasElement> {
        if self.discarded { bail!("discarded flat canvas") }
        Ok(&self.element.as_ref().unwrap())
    }

    pub(super) fn context(&self) -> anyhow::Result<&CanvasRenderingContext2d> {
        if self.discarded { bail!("discarded flat canvas") }
        Ok(&self.context.as_ref().unwrap())
    }

    pub(super) fn discard(&mut self) -> anyhow::Result<()> {
        if self.discarded { return Ok(()); }
        self.element = None;
        self.context = None;
        self.discarded = true;
        Ok(())
    }
}

impl Drop for CanvasElement {
    fn drop(&mut self) {
        self.discard();
    }
}