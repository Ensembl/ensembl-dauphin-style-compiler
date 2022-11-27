use std::sync::Arc;

use peregrine_data::reactive::Observable;
use peregrine_data::{ Colour, DirectColour, DrawnType, Patina, Plotter, SpaceBaseArea, HollowEdge2, SpaceBase, LeafStyle, HotspotPatina };
use peregrine_toolkit::eachorevery::{EachOrEvery, EachOrEveryFilterBuilder};
use peregrine_toolkit::error::Error;
use peregrine_toolkit::log;
use super::directcolourdraw::{DirectProgram, DirectColourDraw};
use super::super::layers::layer::{ Layer };
use super::texture::{xxx_texture_name, TextureDraw};
use crate::shape::canvasitem::heraldry::{HeraldryHandle, HeraldryCanvas, HeraldryScale};
use crate::shape::canvasitem::text::draw_text;
use crate::shape::core::wigglegeometry::{make_wiggle};
use crate::shape::layers::drawing::DynamicShape;
use crate::shape::layers::drawingtools::{DrawingToolsBuilder, CanvasType};
use crate::shape::layers::geometry::{GeometryProcessName };
use crate::shape::layers::patina::{Freedom, PatinaProcessName};
use crate::shape::triangles::rectangles::{Rectangles, RectanglesData, GLAttachmentPoint };
use crate::shape::triangles::drawgroup::DrawGroup;
use crate::shape::util::eoethrow::{eoe_throw2};
use crate::webgl::canvas::composition::canvasitem::{CanvasItemAreaSource, CanvasItemArea};
use crate::webgl::canvas::htmlcanvas::canvasinuse::CanvasInUse;
use crate::webgl::{ ProcessStanzaAddable };
use crate::webgl::global::WebGlGlobal;

#[cfg_attr(debug_assertions,derive(Debug))]
pub(crate) enum SimpleShapePatina {
    Solid(EachOrEvery<DirectColour>),
    Hollow(EachOrEvery<DirectColour>),
    Hotspot(HotspotPatina),
    None
}

fn simplify_colours(colours: &EachOrEvery<Colour>) -> Result<EachOrEvery<DirectColour>,Error> {
    Ok(colours.map_results(|colour| {
        match colour {
            Colour::Direct(d) => Ok(d.clone()),
            _ => Err(Error::fatal("attempt to simplify pattern to colour"))
        }
    })?)
}

impl SimpleShapePatina {
    pub(crate) fn from_patina(patina: &Patina) -> Result<SimpleShapePatina,Error> {
        Ok(match patina {
            Patina::Drawn(drawn_variety,colours) => {
                match drawn_variety {
                    DrawnType::Stroke(_) => SimpleShapePatina::Hollow(simplify_colours(colours)?),
                    DrawnType::Fill => SimpleShapePatina::Solid(simplify_colours(colours)?),
                }
            },
            Patina::Hotspot(hotspot) => { SimpleShapePatina::Hotspot(hotspot.clone()) }
            Patina::Metadata(_,_) => { SimpleShapePatina::None }
        })
    }
}

pub(crate) enum GLShape {
    Text(SpaceBase<f64,LeafStyle>,Option<SpaceBase<f64,()>>,Vec<CanvasItemAreaSource>,EachOrEvery<i8>,DrawGroup,GLAttachmentPoint),
    Image(SpaceBase<f64,LeafStyle>,Vec<CanvasItemAreaSource>,EachOrEvery<i8>,DrawGroup),
    Heraldry(SpaceBaseArea<f64,LeafStyle>,EachOrEvery<HeraldryHandle>,EachOrEvery<i8>,DrawGroup,HeraldryCanvas,HeraldryScale,Option<HollowEdge2<f64>>,Option<SpaceBaseArea<Observable<'static,f64>,()>>),
    Wiggle((f64,f64),Arc<Vec<Option<f64>>>,Plotter,i8),
    SpaceBaseRect(SpaceBaseArea<f64,LeafStyle>,SimpleShapePatina,EachOrEvery<i8>,DrawGroup,Option<SpaceBaseArea<Observable<'static,f64>,()>>),
    Circle(SpaceBase<f64,LeafStyle>,EachOrEvery<f64>,SimpleShapePatina,EachOrEvery<i8>,DrawGroup,Option<SpaceBase<Observable<'static,f64>,()>>)
}

fn add_colour(addable: &mut dyn ProcessStanzaAddable, patina: &SimpleShapePatina, draw: &DirectColourDraw, count: usize) -> Result<(),Error> {
    let vertexes = match patina {
        SimpleShapePatina::Solid(_) => 4,
        SimpleShapePatina::Hollow(_) => 8,
        _ => 0
    };
    match patina {
        SimpleShapePatina::Solid(colours) |
        SimpleShapePatina::Hollow(colours) => {
            draw.direct(addable,&colours,vertexes,count)?;
        },
        _ => {}
    }
    Ok(())
}

pub(crate) fn dims_to_sizes(areas: &[CanvasItemArea], factor: f64) -> (Vec<f64>,Vec<f64>) {
    let mut x_sizes = vec![];
    let mut y_sizes = vec![];
    for dim in areas {
        let size = dim.size();
        x_sizes.push(size.0 as f64 * factor);
        y_sizes.push(size.1 as f64 * factor);
    }
    (x_sizes,y_sizes)
}

fn draw_area_from_canvas(layer: &mut Layer, gl: &mut WebGlGlobal, draw_group: &DrawGroup, area: &SpaceBaseArea<f64,LeafStyle>, depth: &EachOrEvery<i8>, canvas: &CanvasInUse, dims: &[CanvasItemArea], free: bool, edge: &Option<HollowEdge2<f64>>, freedom: &Freedom, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Result<Box<dyn DynamicShape>,Error> {
    let left = layer.left();
    let patina_name = xxx_texture_name(canvas,freedom);
    let builder = layer.get_process_builder(&GeometryProcessName::Triangles(draw_group.geometry().clone()),&patina_name)?;
    let draw = TextureDraw::new(builder,freedom)?;
    let mut rectangles = RectanglesData::new_area(builder,&area,depth,left,false,&draw_group,edge,wobble)?;
    let campaign = rectangles.elements_mut();
    draw.add_rectangle(campaign,&canvas,&dims,freedom)?;
    campaign.close()?;
    Ok(Box::new(Rectangles::new(rectangles,gl)))
}

pub(crate) fn draw_points_from_canvas2(layer: &mut Layer, gl: &mut WebGlGlobal, draw_group: &DrawGroup, points: &SpaceBase<f64,LeafStyle>, run: &Option<SpaceBase<f64,()>>, x_sizes: Vec<f64>, y_sizes:Vec<f64>, depth: &EachOrEvery<i8>, canvas: &CanvasInUse, dims: &[CanvasItemArea], freedom: &Freedom, attachment: GLAttachmentPoint, wobble: Option<SpaceBase<Observable<'static,f64>,()>>) -> Result<Box<dyn DynamicShape>,Error> {
    let left = layer.left();
    let patina_name = xxx_texture_name(canvas,freedom);
    let builder = layer.get_process_builder(&GeometryProcessName::Triangles(draw_group.geometry().clone()),&patina_name)?;
    let draw = TextureDraw::new(builder,freedom)?;
    let mut rectangles = RectanglesData::new_sized(builder,&points,run,x_sizes,y_sizes,depth,left,false,&draw_group,attachment,wobble)?;
    let campaign = rectangles.elements_mut();
    draw.add_rectangle(campaign,&canvas,&dims,&Freedom::None)?;
    campaign.close()?;
    Ok(Box::new(Rectangles::new(rectangles,gl)))
}

fn draw_heraldry_canvas(layer: &mut Layer, gl: &mut WebGlGlobal, tools: &mut DrawingToolsBuilder, kind: &DrawGroup, area_a: &SpaceBaseArea<f64,LeafStyle>, handles: &EachOrEvery<HeraldryHandle>, depth: &EachOrEvery<i8>, heraldry_canvas: &HeraldryCanvas, scale: &HeraldryScale, edge: &Option<HollowEdge2<f64>>, count: usize, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Result<Option<Box<dyn DynamicShape>>,Error> {
    let mut dims = vec![];
    let mut filter_builder = EachOrEveryFilterBuilder::new();
    for (i,handle) in eoe_throw2("heraldry",handles.iter(count))?.enumerate() {
        if let Some(handle) = handle.get_texture_area_on_bitmap(&heraldry_canvas) {
            let area = handle.get()?;
            dims.push(area);
            filter_builder.set(i);
        }
    }
    let filter = filter_builder.make(area_a.len());
    if filter.count() == 0 { return Ok(None); }
    let canvas = tools.composition_builder(&heraldry_canvas.to_canvas_type()).canvas().ok_or_else(|| Error::fatal("no canvas id A"))?;
    Ok(Some(draw_area_from_canvas(layer,gl,kind,&area_a.filter(&filter),&depth.filter(&filter),&canvas,&dims,scale.is_free(),edge,&heraldry_canvas.to_freedom(),wobble)?))
}

pub(crate) enum ShapeToAdd {
    Dynamic(Box<dyn DynamicShape>),
    Hotspot(SpaceBaseArea<f64,LeafStyle>,HotspotPatina),
    None
}

pub(crate) fn add_shape_to_layer(layer: &mut Layer, gl: &mut WebGlGlobal, tools: &mut DrawingToolsBuilder, shape: GLShape) -> Result<ShapeToAdd,Error> {
    let bitmap_multiplier = gl.refs().canvas_source.bitmap_multiplier() as f64;
    match shape {
        GLShape::Wiggle((start,end),yy,Plotter(_,colour),depth) => {
            let left = layer.left();
            let process = layer.get_process_builder(&GeometryProcessName::Wiggle,&PatinaProcessName::Direct)?;
            let (mut array,count) = make_wiggle(process,start,end,&yy,left,depth)?;
            let draw = DirectColourDraw::new(&DirectProgram::new(process.program_builder())?)?;
            draw.direct(&mut array,&EachOrEvery::every(colour),1,count)?;
            array.close()?;
            Ok(ShapeToAdd::None)
        },
        GLShape::Circle(position,radius,patina,depth,draw_group,wobble) => {
            let left = layer.left();
            log!("circle {:?}",position);
            //make_circle(layer,&GeometryProcessName::Triangles(draw_group.geometry().clone()),&mut patina_yielder,position,radius,depth,left,&draw_group)?;
            Ok(ShapeToAdd::None)
        },
        GLShape::Text(points,run,handles,depth,draw_group,attachment) => {
            draw_text(layer,gl,tools,points,run,&handles,depth,&draw_group,attachment)
        },
        GLShape::Image(points,handles,depth,kind) => {
            let bitmap_dims = handles.iter()
                .map(|handle| handle.get())
                .collect::<Result<Vec<_>,_>>()?;
            if bitmap_dims.len() == 0 { return Ok(ShapeToAdd::None); }
            let (x_sizes,y_sizes) = dims_to_sizes(&bitmap_dims,1./bitmap_multiplier);
            let canvas = tools.composition_builder(&CanvasType::Crisp).canvas().ok_or_else(|| Error::fatal("no canvas id A"))?;
            let rectangles = draw_points_from_canvas2(layer,gl,&kind,&points,&None,x_sizes,y_sizes,&depth,&canvas,&bitmap_dims,&Freedom::None,GLAttachmentPoint::Left,None)?;
            Ok(ShapeToAdd::Dynamic(rectangles))
        },
        GLShape::Heraldry(area,handles,depth,kind,heraldry_canvas,scale,edge,wobble) => {
            let rectangles = draw_heraldry_canvas(layer,gl,tools,&kind,&area,&handles,&depth,&heraldry_canvas,&scale,&edge,area.len(),wobble)?;
            if let Some(rectangles) = rectangles {
                Ok(ShapeToAdd::Dynamic(rectangles))
            } else {
                Ok(ShapeToAdd::None)
            }
        },
        GLShape::SpaceBaseRect(area,simple_shape_patina,depth,draw_group,wobble) => {
            let left = layer.left();
            match simple_shape_patina {
                SimpleShapePatina::Solid(_) | SimpleShapePatina::Hollow(_) => {
                    let hollow = match simple_shape_patina { SimpleShapePatina::Hollow(_) => true, _ => false };
                    let builder = layer.get_process_builder(&GeometryProcessName::Triangles(draw_group.geometry().clone()),&PatinaProcessName::Direct)?;
                    let mut rectangles = RectanglesData::new_area(builder,&area,&depth,left,hollow,&draw_group,&None,wobble)?;
                    let draw = DirectColourDraw::new(&DirectProgram::new(builder.program_builder())?)?;
                    let campaign = rectangles.elements_mut();
                    add_colour(campaign,&simple_shape_patina,&draw,area.len())?;
                    campaign.close()?;
                    Ok(ShapeToAdd::Dynamic(Box::new(Rectangles::new(rectangles,&gl))))
                },
                SimpleShapePatina::Hotspot(hotspot) => {
                    Ok(ShapeToAdd::Hotspot(area,hotspot))
                },
                _ => {
                    Ok(ShapeToAdd::None)
                }
            }
        }
    }
}
