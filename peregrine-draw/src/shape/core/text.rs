use peregrine_data::{ DirectColour, PenGeometry, Background, LeafStyle, TextShape };
use keyed::keyed_handle;
use peregrine_toolkit::lock;
use crate::shape::layers::drawingtools::DrawingToolsBuilder;
use crate::shape::triangles::drawgroup::DrawGroup;
use crate::util::fonts::Fonts;
use crate::webgl::canvas::flatplotallocator::FlatPositionManager;
use crate::webgl::{ CanvasWeave, Flat };
use crate::webgl::global::WebGlGlobal;
use super::drawshape::GLShape;
use super::flatdrawing::{FlatDrawingItem, FlatDrawingManager};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use crate::util::message::Message;

struct StructuredText {
    pre_text: String,
    text: String
}

impl StructuredText {
    fn new(src: &str) -> StructuredText {
        let mut src = src.to_string();
        let mut pre_text = String::new();
        /* Remove any pre-text */
        if let Some(index) = src.find("\0<") {
            pre_text = src.split_off(index);
            (src,pre_text) = (pre_text[2..].to_string(),src);
        }
        StructuredText {
            pre_text, 
            text: src
        }
    }
}

keyed_handle!(TextHandle);

const PAD : u32 = 4;

fn pad(x: (u32,u32)) -> (u32,u32) {
    (x.0+PAD,x.1+PAD)
}

// XXX dedup from flat: generally move all text stuff into here
fn pen_to_font(pen: &PenGeometry, bitmap_multiplier: f64) -> String {
    format!("{}px {}",(pen.size_in_webgl() * bitmap_multiplier).round(),pen.name())
}

pub(crate) struct Text {
    pen: PenGeometry,
    text: String,
    colour: DirectColour,
    background: Option<Background>
}

impl Text {
    fn new(pen: &PenGeometry, text: &str, colour: &DirectColour, background: &Option<Background>) -> Text {
        Text { pen: pen.clone(), text: text.to_string(), colour: colour.clone(), background: background.clone() }
    }

    async fn prepare(&self, fonts: &Fonts, bitmap_multiplier: f64) {
        let new_font = pen_to_font(&self.pen,bitmap_multiplier);
        fonts.load_font(&new_font).await;
    }
}

impl FlatDrawingItem for Text {
    fn calc_size(&mut self, gl: &mut WebGlGlobal) -> Result<(u32,u32),Message> {
        let gl_ref = gl.refs();
        let document = gl_ref.document.clone();
        let canvas = gl_ref.flat_store.scratch(&document,&CanvasWeave::Crisp,(100,100))?;
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

    fn group_hash(&self) -> Option<u64> {
        Some(self.pen.group_hash())
    }

    fn build(&mut self, canvas: &mut Flat, text_origin: (u32,u32), size: (u32,u32)) -> Result<(),Message> {
        canvas.set_font(&self.pen)?;
        let background = self.background.clone().unwrap_or_else(|| Background::none());
        canvas.text(&self.text,pad(text_origin),size,&self.colour,&background)?;
        Ok(())
    }
}

pub struct DrawingText(FlatDrawingManager<TextHandle,Text>,Fonts,f64);

impl DrawingText {
    pub(crate) fn new(fonts: &Fonts, bitmap_multiplier: f64) -> DrawingText {
        DrawingText(FlatDrawingManager::new(),fonts.clone(),bitmap_multiplier)
    }

    pub fn add_text(&mut self, pen: &PenGeometry, text: &str, colour: &DirectColour, background: &Option<Background>) -> TextHandle {
        self.0.add(Text::new(pen,text,colour,background))
    }

    pub(crate) async fn calculate_requirements(&mut self, gl: &Arc<Mutex<WebGlGlobal>>, allocator: &mut FlatPositionManager) -> Result<(),Message> {
        for text in self.0.iter_mut() {
            text.prepare(&self.1,self.2).await;
        }
        self.0.calculate_requirements(&mut *lock!(gl),allocator)
    }

    pub(crate) fn manager(&mut self) -> &mut FlatDrawingManager<TextHandle,Text> { &mut self.0 }
}

pub(super) fn add_text(out: &mut Vec<GLShape>, tools: &mut DrawingToolsBuilder, shape: &TextShape<LeafStyle>, draw_group: &DrawGroup, gl: &mut WebGlGlobal) {
    let depth = shape.position().allotments().map(|x| x.depth);
    let drawing_text = tools.text();
    let background = shape.pen().background();
    let texts = shape.iter_texts().collect::<Vec<_>>();
    let colours_iter = shape.pen().colours().iter(texts.len()).unwrap();
    let mut pretexts = vec![];
    let mut handles = vec![];
    for (text,colour) in texts.iter().zip(colours_iter) {
        let structured = StructuredText::new(text);
        let id = drawing_text.add_text(&shape.pen().geometry(),&structured.text,colour,background);
        let mut pretext_x = 0.;
        if structured.pre_text != "" {
            let mut pretext = Text::new(shape.pen().geometry(),&structured.pre_text,colour,background);
            if let Ok((x,_)) = pretext.calc_size(gl) {
                pretext_x = x as f64 / (gl.device_pixel_ratio() as f64);
            }
        }
        pretexts.push(pretext_x);
        handles.push(id);
    }
    let mut positions = shape.position().clone();
    if pretexts.len() > 0 {
        positions.fold_tangent(&pretexts,|a,b| a+b); // unwrap ok cos cycle and > 0.
    }
    let xxx = positions.major().clone();
    out.push(GLShape::Text(xxx,handles,depth,draw_group.clone()));
}
