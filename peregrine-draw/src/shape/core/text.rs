use peregrine_data::{ Pen, DirectColour };
use keyed::keyed_handle;
use crate::webgl::canvas::flatplotallocator::FlatPositionManager;
use crate::webgl::{ CanvasWeave, FlatId, FlatStore, Flat };
use crate::webgl::global::WebGlGlobal;
use super::flatdrawing::{FlatDrawingItem, FlatDrawingManager};
use super::texture::{CanvasTextureArea };
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use crate::util::message::Message;

// TODO padding measurements!

keyed_handle!(TextHandle);

const PAD : u32 = 4;

fn pad(x: (u32,u32)) -> (u32,u32) {
    (x.0+PAD,x.1+PAD)
}

pub(crate) struct Text {
    pen: Pen,
    text: String,
    colour: DirectColour
}

impl Text {
    fn new(pen: &Pen, text: &str, colour: &DirectColour) -> Text {
        Text { pen: pen.clone(), text: text.to_string(), colour: colour.clone() }
    }
}

impl FlatDrawingItem for Text {
    fn calc_size(&mut self, gl: &mut WebGlGlobal) -> Result<(u32,u32),Message> {
        let document = gl.document().clone();
        let canvas = gl.canvas_store_mut().scratch(&document,&CanvasWeave::Crisp,(100,100))?;
        canvas.set_font(&self.pen)?;
        canvas.measure(&self.text)
    }

    fn padding(&mut self, _: &mut WebGlGlobal) -> Result<(u32,u32),Message> { Ok((PAD,PAD)) }

    fn compute_hash(&self) -> Option<u64> {
        let mut hasher = DefaultHasher::new();
        self.pen.hash(&mut hasher);
        self.text.hash(&mut hasher);
        self.colour.hash(&mut hasher);
        Some(hasher.finish())
    }

    fn build(&mut self, canvas: &mut Flat, text_origin: (u32,u32), mask_origin: (u32,u32), size: (u32,u32)) -> Result<(),Message> {
        canvas.set_font(&self.pen)?;
        canvas.text(&self.text,pad(text_origin),size,&self.colour)?;
        canvas.text(&self.text,pad(mask_origin),size,&DirectColour(0,0,0,255))?;
        Ok(())
    }
}

pub struct DrawingText(FlatDrawingManager<TextHandle,Text>);

impl DrawingText {
    pub fn new() -> DrawingText { DrawingText(FlatDrawingManager::new()) }

    pub fn add_text(&mut self, pen: &Pen, text: &str, colour: &DirectColour) -> TextHandle {
        self.0.add(Text::new(pen,text,colour))
    }

    pub(crate) fn calculate_requirements(&mut self, gl: &mut WebGlGlobal, allocator: &mut FlatPositionManager) -> Result<(),Message> {
        self.0.calculate_requirements(gl,allocator,|vv| {
            /* sort by pen to speed up calculation */
            let mut texts_by_pen = HashMap::new();
            for v in vv.drain(..) {
                texts_by_pen.entry(v.0.pen.clone()).or_insert_with(|| vec![]).push(v);
            }
            let mut out = vec![];
            for (_,mut texts) in texts_by_pen.drain() {
                out.append(&mut texts);
            }
            *vv = out;
        })
    }

    pub(crate) fn manager(&mut self) -> &mut FlatDrawingManager<TextHandle,Text> { &mut self.0 }
}
