use peregrine_data::{ DirectColour, PenGeometry, Background, LeafStyle, TextShape, SpaceBase };
use keyed::keyed_handle;
use peregrine_toolkit::eachorevery::EachOrEvery;
use peregrine_toolkit::lock;
use crate::shape::layers::drawingtools::DrawingToolsBuilder;
use crate::shape::layers::layer::Layer;
use crate::shape::triangles::drawgroup::DrawGroup;
use crate::util::fonts::Fonts;
use crate::webgl::canvas::flatplotallocator::FlatPositionManager;
use crate::webgl::canvas::structuredtext::StructuredText;
use crate::webgl::{ CanvasWeave, Flat };
use crate::webgl::global::WebGlGlobal;
use super::drawshape::{GLShape, ShapeToAdd, dims_to_sizes, draw_points_from_canvas2};
use super::flatdrawing::{FlatDrawingItem, FlatDrawingManager};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use crate::util::message::Message;

keyed_handle!(TextHandle);

const PAD : u32 = 4;

fn pad(x: (u32,u32)) -> (u32,u32) {
    (x.0+PAD,x.1+PAD)
}

pub(crate) struct Text {
    text: StructuredText,
}

impl Text {
    fn new(pen: &PenGeometry, text: &str, colour: &DirectColour, background: &Option<Background>) -> Text {
        Text {
            text: StructuredText::new(pen,text,colour,background)
        }
    }

    async fn prepare(&self, fonts: &Fonts, bitmap_multiplier: f64) {
        self.text.prepare(fonts,bitmap_multiplier).await;
    }
}

impl FlatDrawingItem for Text {
    fn calc_size(&mut self, gl: &mut WebGlGlobal) -> Result<(u32,u32),Message> {
        let gl_ref = gl.refs();
        let document = gl_ref.document.clone();
        let canvas = gl_ref.flat_store.scratch(&document,&CanvasWeave::Crisp,(100,100))?;
        self.text.measure(canvas)
    }

    fn padding(&mut self, _: &mut WebGlGlobal) -> Result<(u32,u32),Message> { Ok((PAD,PAD)) }

    fn compute_hash(&self) -> Option<u64> {
        let mut hasher = DefaultHasher::new();
        self.text.hash(&mut hasher);
        Some(hasher.finish())
    }

    fn group_hash(&self) -> Option<u64> {
        let mut hasher = DefaultHasher::new();
        self.text.group().hash(&mut hasher);
        Some(hasher.finish())
    }

    fn build(&mut self, canvas: &mut Flat, text_origin: (u32,u32), size: (u32,u32)) -> Result<(),Message> {
        self.text.draw(canvas,text_origin,size)
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

pub(super) fn prepare_text(out: &mut Vec<GLShape>, tools: &mut DrawingToolsBuilder, shape: &TextShape<LeafStyle>, draw_group: &DrawGroup, gl: &mut WebGlGlobal) {
    let depth = shape.position().allotments().map(|x| x.depth);
    let drawing_text = tools.text();
    let background = shape.pen().background();
    let texts = shape.iter_texts().collect::<Vec<_>>();
    let colours_iter = shape.pen().colours().iter(texts.len()).unwrap();
    let mut handles = vec![];
    for (text,colour) in texts.iter().zip(colours_iter) {
        let id = drawing_text.add_text(&shape.pen().geometry(),&text,colour,background);
        handles.push(id);
    }
    let mut positions = shape.position().clone();
    out.push(GLShape::Text(positions,shape.run().cloned(),handles,depth,draw_group.clone()));
}

pub(super) fn draw_text(layer: &mut Layer, gl: &mut WebGlGlobal, tools: &mut DrawingToolsBuilder,
                    points: SpaceBase<f64,LeafStyle>,
                    run: Option<SpaceBase<f64,()>>,
                    handles: &[TextHandle], depth: EachOrEvery<i8>, draw_group: &DrawGroup
                ) -> Result<ShapeToAdd,Message> {
    let bitmap_multiplier = gl.refs().flat_store.bitmap_multiplier() as f64;
    let text = tools.text();
    let bitmap_dims = handles.iter()
        .map(|handle| text.manager().get_texture_areas_on_bitmap(handle))
        .collect::<Result<Vec<_>,_>>()?;
    if bitmap_dims.len() == 0 { return Ok(ShapeToAdd::None); }
    let (x_sizes,y_sizes) = dims_to_sizes(&bitmap_dims,1./bitmap_multiplier);
    let canvas = text.manager().canvas_id().ok_or_else(|| Message::CodeInvariantFailed("no canvas id A".to_string()))?;
    let rectangles = draw_points_from_canvas2(layer,gl,&draw_group,&points,&run,x_sizes,y_sizes,&depth,&canvas,&bitmap_dims,false,None)?;
    Ok(ShapeToAdd::Dynamic(rectangles))
}
