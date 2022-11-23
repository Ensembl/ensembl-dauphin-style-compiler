use peregrine_data::{ DirectColour, PenGeometry, Background, LeafStyle, TextShape, SpaceBase };
use peregrine_toolkit::eachorevery::EachOrEvery;
use peregrine_toolkit::error::Error;
use crate::shape::layers::drawingtools::{DrawingToolsBuilder, CanvasType};
use crate::shape::layers::layer::Layer;
use crate::shape::layers::patina::Freedom;
use crate::shape::triangles::drawgroup::DrawGroup;
use crate::shape::triangles::rectangles::GLAttachmentPoint;
use crate::util::fonts::Fonts;
use crate::webgl::canvas::structuredtext::StructuredText;
use crate::webgl::canvas::composition::canvasitem::{CanvasItemSize, CanvasItemAreaSource, CanvasItem};
use crate::webgl::{ CanvasWeave, CanvasAndContext };
use crate::webgl::global::WebGlGlobal;
use super::drawshape::{GLShape, ShapeToAdd, dims_to_sizes, draw_points_from_canvas2};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc};

const PAD : u32 = 4;

#[derive(Clone)]
pub(crate) struct Text {
    text: Arc<StructuredText>,
}

impl Text {
    fn new(pen: &PenGeometry, text: &str, colour: &DirectColour, background: &Option<Background>) -> Text {
        Text {
            text: Arc::new(StructuredText::new(pen,text,colour,background))
        }
    }

    async fn prepare(&self, fonts: &Fonts, bitmap_multiplier: f64) {
        self.text.prepare(fonts,bitmap_multiplier).await;
    }
}

impl CanvasItem for Text {
    fn calc_size(&self, gl: &mut WebGlGlobal) -> Result<CanvasItemSize,Error> {
        let gl_ref = gl.refs();
        let mut canvas = gl_ref.scratch_canvases.scratch(&CanvasWeave::Crisp,(100,100))?;
        Ok(CanvasItemSize::new( self.text.measure(canvas.get_mut())?,(PAD,PAD)))
    }

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

    fn draw_on_bitmap(&self, canvas: &mut CanvasAndContext, text_origin: (u32,u32), size: (u32,u32)) -> Result<(),Error> {
        self.text.draw(canvas,text_origin,size)
    }
}

pub struct DrawingText(Vec<Text>,Fonts,f64);

impl DrawingText {
    pub(crate) fn new(fonts: &Fonts, bitmap_multiplier: f64) -> DrawingText {
        DrawingText(vec![],fonts.clone(),bitmap_multiplier)
    }

    fn make(&mut self, pen: &PenGeometry, text: &str, colour: &DirectColour, background: &Option<Background>) -> Text {
        let text = Text::new(pen,text,colour,background);
        self.0.push(text.clone());
        text
    }

    pub(crate) async fn prepare_for_allocation(&self) -> Result<(),Error> {
        for text in self.0.iter() {
            text.prepare(&self.1,self.2).await;
        }
        Ok(())
    }
}

pub(super) fn prepare_text(out: &mut Vec<GLShape>, tools: &mut DrawingToolsBuilder, shape: &TextShape<LeafStyle>, draw_group: &DrawGroup) -> Result<(),Error> {
    let depth = shape.position().allotments().map(|x| x.depth);
    let drawing_text = tools.text();
    let background = shape.pen().background();
    let texts = shape.iter_texts().collect::<Vec<_>>();
    let colours_iter = shape.pen().colours().iter(texts.len()).unwrap();
    let mut all_texts = vec![];
    for (text,colour) in texts.iter().zip(colours_iter) {
        let item = drawing_text.make(&shape.pen().geometry(),&text,colour,background);
        all_texts.push(item);
    }
    drop(drawing_text);
    let manager = tools.composition_builder(&CanvasType::Crisp);
    let handles = all_texts.drain(..).map(|x| manager.add(x)).collect::<Result<Vec<_>,_>>()?;
    let positions = shape.position().clone();
    out.push(GLShape::Text(positions,shape.run().cloned(),handles,depth,draw_group.clone(),GLAttachmentPoint::new(shape.pen().attachment())));
    Ok(())
}

pub(super) fn draw_text(layer: &mut Layer, gl: &mut WebGlGlobal, tools: &mut DrawingToolsBuilder,
                    points: SpaceBase<f64,LeafStyle>,
                    run: Option<SpaceBase<f64,()>>,
                    handles: &[CanvasItemAreaSource], depth: EachOrEvery<i8>, draw_group: &DrawGroup,
                    attachment: GLAttachmentPoint,
                ) -> Result<ShapeToAdd,Error> {
    let bitmap_multiplier = gl.refs().canvas_source.bitmap_multiplier() as f64;
    let bitmap_dims = handles.iter()
        .map(|handle| handle.get())
        .collect::<Result<Vec<_>,_>>()?;
    if bitmap_dims.len() == 0 { return Ok(ShapeToAdd::None); }
    let (x_sizes,y_sizes) = dims_to_sizes(&bitmap_dims,1./bitmap_multiplier);
    let canvas = tools.composition_builder(&CanvasType::Crisp).canvas().ok_or_else(|| Error::fatal("no canvas id A"))?;
    let rectangles = draw_points_from_canvas2(layer,gl,&draw_group,&points,&run,x_sizes,y_sizes,&depth,&canvas,&bitmap_dims,&Freedom::None,attachment,None)?;
    Ok(ShapeToAdd::Dynamic(rectangles))
}
