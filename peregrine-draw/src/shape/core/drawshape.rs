use std::sync::Arc;

use peregrine_data::reactive::Observable;
use peregrine_data::{ Colour, DirectColour, DrawnType, Patina, Plotter, SpaceBaseArea, HollowEdge2, SpaceBase, LeafStyle, HotspotPatina };
use peregrine_toolkit::eachorevery::{EachOrEvery, EachOrEveryFilterBuilder};
use super::directcolourdraw::DirectYielder;
use super::flatdrawing::CanvasItemHandle;
use super::spotcolourdraw::SpotColourYielder;
use super::text::draw_text;
use super::super::layers::layer::{ Layer };
use super::texture::{CanvasTextureArea, TextureYielder};
use crate::shape::core::wigglegeometry::{make_wiggle};
use crate::shape::heraldry::heraldry::{HeraldryCanvas, HeraldryHandle, HeraldryScale};
use crate::shape::layers::drawing::DynamicShape;
use crate::shape::layers::drawingtools::{DrawingToolsBuilder, CanvasType};
use crate::shape::layers::geometry::{GeometryYielder, GeometryProcessName };
use crate::shape::layers::patina::{PatinaYielder, Freedom};
use crate::shape::triangles::rectangles::{Rectangles, RectanglesData, GLAttachmentPoint };
use crate::shape::triangles::drawgroup::DrawGroup;
use crate::shape::util::eoethrow::eoe_throw;
use crate::webgl::{ ProcessStanzaAddable, CanvasInUse };
use crate::webgl::global::WebGlGlobal;
use crate::util::message::Message;

#[cfg_attr(debug_assertions,derive(Debug))]
pub(crate) enum SimpleShapePatina {
    Solid(EachOrEvery<DirectColour>),
    Hollow(EachOrEvery<DirectColour>),
    SolidSpot(DirectColour),
    HollowSpot(DirectColour),
    Hotspot(HotspotPatina),
    None
}

fn simplify_colours(colours: &EachOrEvery<Colour>) -> Result<EachOrEvery<DirectColour>,Message> {
    Ok(colours.map_results(|colour| {
        match colour {
            Colour::Direct(d) => Ok(d.clone()),
            Colour::Spot(d) => Ok(d.clone()),
            _ => Err(Message::CodeInvariantFailed(format!("attempt to simplify pattern to colour")))
        }
    })?)
}

impl SimpleShapePatina {
    pub(crate) fn from_patina(patina: &Patina) -> Result<SimpleShapePatina,Message> {
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

    pub(crate) fn spot_from_patina(colour: &DirectColour, patina: &Patina) -> Result<SimpleShapePatina,Message> {
        Ok(match patina {
            Patina::Drawn(drawn_variety,_) => {
                match drawn_variety {
                    DrawnType::Stroke(_) => SimpleShapePatina::HollowSpot(colour.clone()),
                    DrawnType::Fill => SimpleShapePatina::SolidSpot(colour.clone()),
                }
            },
            Patina::Hotspot(hotspot) => { SimpleShapePatina::Hotspot(hotspot.clone()) },
            Patina::Metadata(_,_) => { SimpleShapePatina::None }
        })
    }

    fn build(&self) -> DrawingShapePatina {
        match self {
            SimpleShapePatina::Solid(c) => DrawingShapePatina::Solid(DirectYielder::new(),c.clone()),
            SimpleShapePatina::Hollow(c) => DrawingShapePatina::Hollow(DirectYielder::new(),c.clone()),
            SimpleShapePatina::SolidSpot(c) => DrawingShapePatina::SolidSpot(SpotColourYielder::new(c)),
            SimpleShapePatina::HollowSpot(c) => DrawingShapePatina::HollowSpot(SpotColourYielder::new(c)),
            SimpleShapePatina::Hotspot(hotspot) => DrawingShapePatina::Hotspot(hotspot.clone()),
            SimpleShapePatina::None => DrawingShapePatina::None
        }
    }
}

enum DrawingShapePatina {
    Solid(DirectYielder,EachOrEvery<DirectColour>),
    Hollow(DirectYielder,EachOrEvery<DirectColour>),
    SolidSpot(SpotColourYielder),
    HollowSpot(SpotColourYielder),
    Hotspot(HotspotPatina),
    None
}

enum PatinaTarget<'a> {
    Visual(&'a mut dyn PatinaYielder),
    HotSpot(HotspotPatina),
    None
}

impl DrawingShapePatina {
    pub(crate) fn yielder_mut(&mut self) -> PatinaTarget {
        match self {
            DrawingShapePatina::Solid(dc,_) => PatinaTarget::Visual(dc),
            DrawingShapePatina::Hollow(dc,_) => PatinaTarget::Visual(dc),
            DrawingShapePatina::SolidSpot(dc) => PatinaTarget::Visual(dc),
            DrawingShapePatina::HollowSpot(dc) => PatinaTarget::Visual(dc),
            DrawingShapePatina::Hotspot(hotspot) => PatinaTarget::HotSpot(hotspot.clone()),
            DrawingShapePatina::None => PatinaTarget::None
        }
    }
}

pub(crate) enum GLShape {
    Text(SpaceBase<f64,LeafStyle>,Option<SpaceBase<f64,()>>,Vec<CanvasItemHandle>,EachOrEvery<i8>,DrawGroup,GLAttachmentPoint),
    Image(SpaceBase<f64,LeafStyle>,Vec<CanvasItemHandle>,EachOrEvery<i8>,DrawGroup),
    Heraldry(SpaceBaseArea<f64,LeafStyle>,EachOrEvery<HeraldryHandle>,EachOrEvery<i8>,DrawGroup,HeraldryCanvas,HeraldryScale,Option<HollowEdge2<f64>>,Option<SpaceBaseArea<Observable<'static,f64>,()>>),
    Wiggle((f64,f64),Arc<Vec<Option<f64>>>,Plotter,i8),
    SpaceBaseRect(SpaceBaseArea<f64,LeafStyle>,SimpleShapePatina,EachOrEvery<i8>,DrawGroup,Option<SpaceBaseArea<Observable<'static,f64>,()>>),
}

fn add_colour(addable: &mut dyn ProcessStanzaAddable, simple_shape_patina: &DrawingShapePatina, count: usize) -> Result<(),Message> {
    let vertexes = match simple_shape_patina {
        DrawingShapePatina::Solid(_,_) => 4,
        DrawingShapePatina::Hollow(_,_) => 8,
        _ => 0
    };
    match simple_shape_patina {
        DrawingShapePatina::Solid(direct,colours) |
        DrawingShapePatina::Hollow(direct,colours) => {
            direct.draw()?.direct(addable,&colours,vertexes,count)?;
        },
        _ => {}
    }
    Ok(())
}

pub(super) fn dims_to_sizes(areas: &[CanvasTextureArea], factor: f64) -> (Vec<f64>,Vec<f64>) {
    let mut x_sizes = vec![];
    let mut y_sizes = vec![];
    for dim in areas {
        let size = dim.size();
        x_sizes.push(size.0 as f64 * factor);
        y_sizes.push(size.1 as f64 * factor);
    }
    (x_sizes,y_sizes)
}

fn draw_area_from_canvas(layer: &mut Layer, gl: &mut WebGlGlobal, draw_group: &DrawGroup, area: &SpaceBaseArea<f64,LeafStyle>, depth: &EachOrEvery<i8>, canvas: &CanvasInUse, dims: &[CanvasTextureArea], free: bool, edge: &Option<HollowEdge2<f64>>, freedom: &Freedom, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Result<Box<dyn DynamicShape>,Message> {
    let mut geometry_yielder = draw_group.geometry_yielder();
    let mut patina_yielder = TextureYielder::new(canvas,freedom);
    let left = layer.left();
    let mut rectangles = RectanglesData::new_area(layer, &mut geometry_yielder, &mut patina_yielder,&area,depth,left,false,&draw_group,edge,wobble)?;
    let campaign = rectangles.elements_mut();
    patina_yielder.draw()?.add_rectangle(campaign,&canvas,&dims,freedom)?;
    campaign.close()?;
    Ok(Box::new(Rectangles::new(rectangles,gl)))
}

pub(super) fn draw_points_from_canvas2(layer: &mut Layer, gl: &mut WebGlGlobal, draw_group: &DrawGroup, points: &SpaceBase<f64,LeafStyle>, run: &Option<SpaceBase<f64,()>>, x_sizes: Vec<f64>, y_sizes:Vec<f64>, depth: &EachOrEvery<i8>, canvas: &CanvasInUse, dims: &[CanvasTextureArea], freedom: &Freedom, attachment: GLAttachmentPoint, wobble: Option<SpaceBase<Observable<'static,f64>,()>>) -> Result<Box<dyn DynamicShape>,Message> {
    let mut geometry_yielder = draw_group.geometry_yielder();
    let mut patina_yielder = TextureYielder::new(canvas,freedom);
    let left = layer.left();
    let mut rectangles = RectanglesData::new_sized(layer, &mut geometry_yielder, &mut patina_yielder,&points,run,x_sizes,y_sizes,depth,left,false,&draw_group,attachment,wobble)?;
    let campaign = rectangles.elements_mut();
    patina_yielder.draw()?.add_rectangle(campaign,&canvas,&dims,&Freedom::None)?;
    campaign.close()?;
    Ok(Box::new(Rectangles::new(rectangles,gl)))
}

fn draw_heraldry_canvas(layer: &mut Layer, gl: &mut WebGlGlobal, tools: &mut DrawingToolsBuilder, kind: &DrawGroup, area_a: &SpaceBaseArea<f64,LeafStyle>, handles: &EachOrEvery<HeraldryHandle>, depth: &EachOrEvery<i8>, heraldry_canvas: &HeraldryCanvas, scale: &HeraldryScale, edge: &Option<HollowEdge2<f64>>, count: usize, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Result<Option<Box<dyn DynamicShape>>,Message> {
    let mut dims = vec![];
    let mut filter_builder = EachOrEveryFilterBuilder::new();
    for (i,handle) in eoe_throw("heraldry",handles.iter(count))?.enumerate() {
        if let Some(handle) = handle.get_texture_area_on_bitmap(&heraldry_canvas) {
            let area = handle.drawn_area()?;
            dims.push(area);
            filter_builder.set(i);
        }
    }
    let filter = filter_builder.make(area_a.len());
    if filter.count() == 0 { return Ok(None); }
    let canvas = tools.manager(&heraldry_canvas.to_canvas_type()).canvas_id().ok_or_else(|| Message::CodeInvariantFailed("no canvas id A".to_string()))?;
    Ok(Some(draw_area_from_canvas(layer,gl,kind,&area_a.filter(&filter),&depth.filter(&filter),&canvas,&dims,scale.is_free(),edge,&heraldry_canvas.to_freedom(),wobble)?))
}

pub(crate) enum ShapeToAdd {
    Dynamic(Box<dyn DynamicShape>),
    Hotspot(SpaceBaseArea<f64,LeafStyle>,HotspotPatina),
    None
}

pub(crate) fn add_shape_to_layer(layer: &mut Layer, gl: &mut WebGlGlobal, tools: &mut DrawingToolsBuilder, shape: GLShape) -> Result<ShapeToAdd,Message> {
    let bitmap_multiplier = gl.refs().canvas_source.bitmap_multiplier() as f64;
    match shape {
        GLShape::Wiggle((start,end),yy,Plotter(_,colour),depth) => {
            let mut geometry_yielder = GeometryYielder::new(GeometryProcessName::Wiggle);
            let mut patina_yielder = DirectYielder::new();
            let left = layer.left();
            let (mut array,count) = make_wiggle(layer,&mut geometry_yielder,&mut patina_yielder,start,end,&yy,left,depth)?;
            patina_yielder.draw()?.direct(&mut array, &EachOrEvery::every(colour),1,count)?;
            array.close()?;
            Ok(ShapeToAdd::None)
        },
        GLShape::Text(points,run,handles,depth,draw_group,attachment) => {
            draw_text(layer,gl,tools,points,run,&handles,depth,&draw_group,attachment)
        },
        GLShape::Image(points,handles,depth,kind) => {
            let bitmap_dims = handles.iter()
                .map(|handle| handle.drawn_area())
                .collect::<Result<Vec<_>,_>>()?;
            if bitmap_dims.len() == 0 { return Ok(ShapeToAdd::None); }
            let (x_sizes,y_sizes) = dims_to_sizes(&bitmap_dims,1./bitmap_multiplier);
            let canvas = tools.manager(&CanvasType::Crisp).canvas_id().ok_or_else(|| Message::CodeInvariantFailed("no canvas id A".to_string()))?;
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
            let mut drawing_shape_patina = simple_shape_patina.build();
            let mut geometry_yielder = draw_group.geometry_yielder();
            let left = layer.left();
            match drawing_shape_patina.yielder_mut() {
                PatinaTarget::Visual(patina_yielder) => {
                    let hollow = match simple_shape_patina { SimpleShapePatina::Hollow(_) | SimpleShapePatina::HollowSpot(_) => true, _ => false };
                    let mut rectangles = RectanglesData::new_area(layer,&mut geometry_yielder,patina_yielder,&area,&depth,left,hollow,&draw_group,&None,wobble)?;
                    let campaign = rectangles.elements_mut();
                    add_colour(campaign,&drawing_shape_patina,area.len())?;
                    campaign.close()?;
                    Ok(ShapeToAdd::Dynamic(Box::new(Rectangles::new(rectangles,&gl))))
                },
                PatinaTarget::HotSpot(hotspot) => {
                    Ok(ShapeToAdd::Hotspot(area,hotspot))
                },
                PatinaTarget::None => {
                    Ok(ShapeToAdd::None)
                }
            }
        }
    }
}
