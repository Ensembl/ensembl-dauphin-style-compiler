use wasm_bindgen::JsCast;
use web_sys::{ Document, HtmlCanvasElement, CanvasRenderingContext2d };
use peregrine_data::{ Pen, DirectColour };
use super::weave::CanvasWeave;
use crate::util::message::Message;

fn pen_to_font(pen: &Pen) -> String {
    format!("{}px {}",pen.1,pen.0)
}

fn colour_to_css(c: &DirectColour) -> String {
    format!("rgb({},{},{})",c.0,c.1,c.2)
}

pub(crate) struct Flat {
    element: Option<HtmlCanvasElement>,
    context: Option<CanvasRenderingContext2d>,
    weave: CanvasWeave,
    font: Option<String>,
    font_height: Option<u32>,
    size: (u32,u32),
    discarded: bool
}

impl Flat {
    pub(super) fn new(document: &Document, weave: &CanvasWeave, size: (u32,u32)) -> Result<Flat,Message> {
        let el = document.create_element("canvas").map_err(|e| Message::ConfusedWebBrowser(format!("cannot create canvas")))?;
        let canvas_el = el.dyn_into::<HtmlCanvasElement>().map_err(|_| Message::ConfusedWebBrowser("could not cast canvas to HtmlCanvasElement".to_string()))?;
        canvas_el.set_width(size.0);
        canvas_el.set_height(size.1);
        //document.body().unwrap().append_child(&canvas_el);
        let context = canvas_el
            .get_context("2d").map_err(|_| Message::Canvas2DFailure("cannot get 2d context".to_string()))?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>().map_err(|_| Message::Canvas2DFailure("cannot get 2d context".to_string()))?;
        Ok(Flat {
            element: Some(canvas_el),
            context: Some(context),
            weave: weave.clone(),
            size,
            font: None,
            font_height: None,
            discarded: false
        })
    }

    pub(crate) fn set_font(&mut self, pen: &Pen) -> Result<(),Message> {
        if self.discarded { return Err(Message::CodeInvariantFailed(format!("set_font on discarded flat canvas"))); }
        let new_font = pen_to_font(pen);
        if let Some(old_font) = &self.font {
            if *old_font == new_font { return Ok(()); }
        }
        self.font = Some(new_font.to_string());
        self.font_height = Some(pen.1 as u32);
        self.context()?.set_font(self.font.as_ref().unwrap());
        Ok(())
    }

    pub(crate) fn measure(&mut self, text: &str) -> Result<(u32,u32),Message> {
        if self.discarded { return Err(Message::CodeInvariantFailed(format!("set_font on discarded flat canvas"))); }
        let width = self.context.as_ref().unwrap().measure_text(text).map_err(|e| Message::Canvas2DFailure(format!("cannot measure text: {:?}",e)))?.width();
        let height = self.font_height.ok_or_else(|| Message::CodeInvariantFailed("no font set before measure".to_string()))?;
        Ok((width as u32,height as u32))
    }

    // TODO white-bgd canvas
    pub(crate) fn text(&self, text: &str, origin: (u32,u32), size: (u32,u32), colour: &DirectColour) -> Result<(),Message> {
        if self.discarded { return Err(Message::CodeInvariantFailed(format!("set_font on discarded flat canvas"))); }
        let context = self.context()?;
//        context.set_font("100px 'Lato'");
        context.set_fill_style(&colour_to_css(&DirectColour(255,255,255)).into()); // TODO background colours for pen
        context.fill_rect(origin.0 as f64, origin.1 as f64, size.0 as f64, size.1 as f64);
        context.set_text_baseline("top");
        context.set_fill_style(&colour_to_css(&colour).into());
        context.fill_text(text,origin.0 as f64,origin.1 as f64).map_err(|e| Message::Canvas2DFailure(format!("fill_text failed: {:?}",e)))?;
        Ok(())
    }

    pub(crate) fn size(&self) -> &(u32,u32) { &self.size }
    pub(crate) fn weave(&self) -> &CanvasWeave { &self.weave }
    pub(crate) fn element(&self) -> Result<&HtmlCanvasElement,Message> {
        if self.discarded { return Err(Message::CodeInvariantFailed(format!("set_font on discarded flat canvas"))); }
        Ok(&self.element.as_ref().unwrap())
    }

    pub(super) fn context(&self) -> Result<&CanvasRenderingContext2d,Message> {
        if self.discarded { return Err(Message::CodeInvariantFailed(format!("set_font on discarded flat canvas"))); }
        Ok(&self.context.as_ref().unwrap())
    }

    pub(super) fn discard(&mut self) -> Result<(),Message> {
        if self.discarded { return Ok(()); }
        self.element = None;
        self.context = None;
        self.font = None;
        self.discarded = true;
        Ok(())
    }
}
