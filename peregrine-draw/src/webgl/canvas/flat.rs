use wasm_bindgen::JsCast;
use web_sys::{ Document, HtmlCanvasElement, CanvasRenderingContext2d };
use peregrine_data::{ Pen, DirectColour };
use super::{canvasstore::HtmlFlatCanvas, weave::CanvasWeave};
use crate::util::message::Message;
use super::canvasstore::CanvasStore;

fn pen_to_font(pen: &Pen) -> String {
    format!("{}px {}",pen.1,pen.0)
}

fn colour_to_css(c: &DirectColour) -> String {
    format!("rgb({},{},{})",c.0,c.1,c.2)
}

pub(crate) struct Flat {
    element: Option<HtmlFlatCanvas>,
    context: Option<CanvasRenderingContext2d>,
    weave: CanvasWeave,
    font: Option<String>,
    font_height: Option<u32>,
    size: (u32,u32),
    discarded: bool
}

impl Flat {
    pub(super) fn new(canvas_store: &mut CanvasStore, document: &Document, weave: &CanvasWeave, size: (u32,u32)) -> Result<Flat,Message> {
        let el = canvas_store.allocate(document, size.0, size.1, weave.round_up())?;
        let context = el.element()
            .get_context("2d").map_err(|_| Message::Canvas2DFailure("cannot get 2d context".to_string()))?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>().map_err(|_| Message::Canvas2DFailure("cannot get 2d context".to_string()))?;
        Ok(Flat {
            size: el.size(),
            element: Some(el),
            context: Some(context),
            weave: weave.clone(),
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

    pub(crate) fn rectangle(&self, origin: (u32,u32), size: (u32,u32), colour: &DirectColour) -> Result<(),Message> {
        if self.discarded { return Err(Message::CodeInvariantFailed(format!("set_font on discarded flat canvas"))); }
        let context = self.context()?;
        context.set_fill_style(&colour_to_css(colour).into()); // TODO background colours for pen
        context.fill_rect(origin.0 as f64, origin.1 as f64, size.0 as f64, size.1 as f64);
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
    pub(crate) fn text(&self, text: &str, origin: (u32,u32), size: (u32,u32), colour: &DirectColour) -> Result<(),Message> {
        if self.discarded { return Err(Message::CodeInvariantFailed(format!("set_font on discarded flat canvas"))); }
        self.rectangle(origin,size,&DirectColour(255,255,255,255))?;
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
