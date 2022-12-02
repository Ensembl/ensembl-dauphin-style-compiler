use std::sync::Arc;

use peregrine_data::reactive::Observable;
use peregrine_data::{ Colour, DirectColour, DrawnType, Patina, Plotter, SpaceBaseArea, HollowEdge2, SpaceBase, AuxLeaf, HotspotPatina, AttachmentPoint, PartialSpaceBase, SpaceBasePoint };
use peregrine_toolkit::eachorevery::{EachOrEvery, EachOrEveryFilterBuilder};
use peregrine_toolkit::error::Error;
use super::directcolourdraw::{DirectColourDraw, ColourFragment};
use super::super::layers::layer::{ Layer };
use super::texture::{TextureDrawFactory};
use super::wigglegeometry::{WiggleAdderFactory};
use crate::shape::canvasitem::heraldry::{HeraldryHandle, HeraldryCanvas, HeraldryScale};
use crate::shape::canvasitem::text::draw_text;
use crate::shape::layers::drawing::DynamicShape;
use crate::shape::layers::drawingtools::{DrawingToolsBuilder, CanvasType};
use crate::shape::layers::patina::Freedom;
use crate::shape::triangles::polygon::{SolidPolygonDataFactory, SolidPolygon};
use crate::shape::triangles::rectangles::{Rectangles, RectanglesDataFactory };
use crate::shape::triangles::drawgroup::DrawGroup;
use crate::shape::util::eoethrow::{eoe_throw2};
use crate::webgl::canvas::composition::canvasitem::{CanvasItemAreaSource, CanvasItemArea};
use crate::webgl::canvas::htmlcanvas::canvasinuse::CanvasInUse;
use crate::webgl::{ ProcessStanzaElements };
use crate::webgl::global::WebGlGlobal;

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub(crate) enum GLAttachmentPoint {
    Left,
    Right
}

impl GLAttachmentPoint {
    pub(crate) fn new(input: &AttachmentPoint) -> GLAttachmentPoint {
        match input {
            AttachmentPoint::Left => GLAttachmentPoint::Left,
            AttachmentPoint::Right => GLAttachmentPoint::Right
        }
    }
}

impl GLAttachmentPoint {
    pub(crate) fn sized_to_rectangle(&self,spacebase: &SpaceBase<f64,AuxLeaf>, size_x: &[f64], size_y: &[f64]) -> Result<SpaceBaseArea<f64,AuxLeaf>,Error> {
        let mut near = spacebase.clone();
        let mut far = spacebase.clone();
        match self {
            GLAttachmentPoint::Left => {
                far.fold_tangent(size_x,|v,z| { *v + z });
                far.fold_normal(size_y,|v,z| { *v + z });        
            }
            GLAttachmentPoint::Right => {
                near.fold_tangent(size_x,|v,z| { *v - z });
                far.fold_normal(size_y,|v,z| { *v + z });        
            }
        }
        let area = eoe_throw2("rl1",SpaceBaseArea::new(
            PartialSpaceBase::from_spacebase(near),
            PartialSpaceBase::from_spacebase(far)))?;
        Ok(area)
    }
}

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
    Text(SpaceBase<f64,AuxLeaf>,Option<SpaceBase<f64,()>>,Vec<CanvasItemAreaSource>,EachOrEvery<i8>,DrawGroup,GLAttachmentPoint),
    Image(SpaceBase<f64,AuxLeaf>,Vec<CanvasItemAreaSource>,EachOrEvery<i8>,DrawGroup),
    Heraldry(SpaceBaseArea<f64,AuxLeaf>,EachOrEvery<HeraldryHandle>,EachOrEvery<i8>,DrawGroup,HeraldryCanvas,HeraldryScale,Option<HollowEdge2<f64>>,Option<SpaceBaseArea<Observable<'static,f64>,()>>),
    Wiggle((f64,f64),Arc<Vec<Option<f64>>>,Plotter,i8),
    SpaceBaseRect(SpaceBaseArea<f64,AuxLeaf>,SimpleShapePatina,EachOrEvery<i8>,DrawGroup,Option<SpaceBaseArea<Observable<'static,f64>,()>>),
    Polygon(SpaceBase<f64,AuxLeaf>,EachOrEvery<f64>,i8,usize,f32,SimpleShapePatina,DrawGroup,Option<SpaceBase<Observable<'static,f64>,()>>)
}

fn add_colour(addable: &mut ProcessStanzaElements, patina: &SimpleShapePatina, draw: &DirectColourDraw) -> Result<(),Error> {
    let points_per_shape = addable.points_per_shape();
    let number_of_shapes = addable.number_of_shapes();
    match patina {
        SimpleShapePatina::Solid(colours) |
        SimpleShapePatina::Hollow(colours) => {
            draw.direct(addable,&colours,points_per_shape,number_of_shapes)?;
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

fn draw_area_from_canvas(layer: &mut Layer, left: f64, gl: &mut WebGlGlobal, draw_group: &DrawGroup, area: &SpaceBaseArea<f64,AuxLeaf>, depth: &EachOrEvery<i8>, canvas: &CanvasInUse, dims: &[CanvasItemArea], edge: &Option<HollowEdge2<f64>>, freedom: &Freedom, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Result<Box<dyn DynamicShape>,Error> {
    let rectangle_factory = RectanglesDataFactory::new(&draw_group);
    let draw_factory = TextureDrawFactory::new(canvas,freedom);
    let builder = layer.get_process_builder(&rectangle_factory,&draw_factory)?;
    let draw = draw_factory.make(builder)?;
    let mut rectangles = rectangle_factory.make_area(builder,&area,depth,left,false,edge,wobble)?;
    let campaign = rectangles.elements_mut();
    draw.add_rectangle(campaign,&canvas,&dims,freedom)?;
    campaign.close()?;
    Ok(Box::new(Rectangles::new(rectangles,gl)))
}

pub(crate) fn draw_points_from_canvas2(layer: &mut Layer, left: f64, gl: &mut WebGlGlobal, draw_group: &DrawGroup, points: &SpaceBase<f64,AuxLeaf>, run: &Option<SpaceBase<f64,()>>, x_sizes: Vec<f64>, y_sizes:Vec<f64>, depth: &EachOrEvery<i8>, canvas: &CanvasInUse, dims: &[CanvasItemArea], freedom: &Freedom, attachment: GLAttachmentPoint, wobble: Option<SpaceBase<Observable<'static,f64>,()>>) -> Result<Box<dyn DynamicShape>,Error> {
    let rectangle_factory = RectanglesDataFactory::new(&draw_group);
    let draw_factory = TextureDrawFactory::new(canvas,freedom);
    let builder = layer.get_process_builder(&rectangle_factory,&draw_factory)?;
    let draw = draw_factory.make(builder)?;
    let mut rectangles = rectangle_factory.make_sized(builder,&points,run,x_sizes,y_sizes,depth,left,false,attachment,wobble)?;
    let campaign = rectangles.elements_mut();
    draw.add_rectangle(campaign,&canvas,&dims,&Freedom::None)?;
    campaign.close()?;
    Ok(Box::new(Rectangles::new(rectangles,gl)))
}

fn draw_heraldry_canvas(layer: &mut Layer, left: f64, gl: &mut WebGlGlobal, tools: &mut DrawingToolsBuilder, kind: &DrawGroup, area_a: &SpaceBaseArea<f64,AuxLeaf>, handles: &EachOrEvery<HeraldryHandle>, depth: &EachOrEvery<i8>, heraldry_canvas: &HeraldryCanvas, scale: &HeraldryScale, edge: &Option<HollowEdge2<f64>>, count: usize, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Result<Option<Box<dyn DynamicShape>>,Error> {
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
    Ok(Some(draw_area_from_canvas(layer,left,gl,kind,&area_a.filter(&filter),&depth.filter(&filter),&canvas,&dims,edge,&heraldry_canvas.to_freedom(),wobble)?))
}

pub(crate) enum ShapeToAdd {
    Dynamic(Box<dyn DynamicShape>),
    Hotspot(SpaceBaseArea<f64,AuxLeaf>,HotspotPatina),
    None
}

pub(crate) fn add_shape_to_layer(layer: &mut Layer, left: f64, gl: &mut WebGlGlobal, tools: &mut DrawingToolsBuilder, shape: GLShape) -> Result<ShapeToAdd,Error> {
    let bitmap_multiplier = gl.refs().canvas_source.bitmap_multiplier() as f64;
    match shape {
        GLShape::Wiggle((start,end),yy,Plotter(_,colour),depth) => {
            let wiggle_factory = WiggleAdderFactory::new();
            let fragment_factory = ColourFragment::new();
            let process = layer.get_process_builder(&wiggle_factory,&fragment_factory)?;
            let adder = wiggle_factory.make(process)?;
            let draw = fragment_factory.make(process)?;
            let (mut array,count) = adder.add_wiggle(process,start,end,&yy,left,depth)?;
            draw.direct(&mut array,&EachOrEvery::every(colour),1,count)?;
            array.close()?;
            Ok(ShapeToAdd::None)
        },
        GLShape::Text(points,run,handles,depth,draw_group,attachment) => {
            draw_text(layer,left,gl,tools,points,run,&handles,depth,&draw_group,attachment)
        },
        GLShape::Image(points,handles,depth,kind) => {
            let bitmap_dims = handles.iter()
                .map(|handle| handle.get())
                .collect::<Result<Vec<_>,_>>()?;
            if bitmap_dims.len() == 0 { return Ok(ShapeToAdd::None); }
            let (x_sizes,y_sizes) = dims_to_sizes(&bitmap_dims,1./bitmap_multiplier);
            let canvas = tools.composition_builder(&CanvasType::Crisp).canvas().ok_or_else(|| Error::fatal("no canvas id A"))?;
            let rectangles = draw_points_from_canvas2(layer,left,gl,&kind,&points,&None,x_sizes,y_sizes,&depth,&canvas,&bitmap_dims,&Freedom::None,GLAttachmentPoint::Left,None)?;
            Ok(ShapeToAdd::Dynamic(rectangles))
        },
        GLShape::Heraldry(area,handles,depth,kind,heraldry_canvas,scale,edge,wobble) => {
            let rectangles = draw_heraldry_canvas(layer,left,gl,tools,&kind,&area,&handles,&depth,&heraldry_canvas,&scale,&edge,area.len(),wobble)?;
            if let Some(rectangles) = rectangles {
                Ok(ShapeToAdd::Dynamic(rectangles))
            } else {
                Ok(ShapeToAdd::None)
            }
        },
        GLShape::SpaceBaseRect(area,simple_shape_patina,depth,draw_group,wobble) => {
            match simple_shape_patina {
                SimpleShapePatina::Solid(_) | SimpleShapePatina::Hollow(_) => {
                    let hollow = match simple_shape_patina { SimpleShapePatina::Hollow(_) => true, _ => false };
                    let vertex_factory = RectanglesDataFactory::new(&draw_group);
                    let fragment_factory = ColourFragment::new();
                    let builder = layer.get_process_builder(&vertex_factory,&fragment_factory)?;
                    let mut rectangles = vertex_factory.make_area(builder,&area,&depth,left,hollow,&None,wobble)?;
                    let draw = fragment_factory.make(builder)?;
                    let campaign = rectangles.elements_mut();
                    add_colour(campaign,&simple_shape_patina,&draw)?;
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
        },
        GLShape::Polygon(centre,radius,depth,points,angle,patina,group,wobble) => {
            match patina {
                SimpleShapePatina::Solid(_) => {
                    let vertex_factory = SolidPolygonDataFactory::new(&group);
                    let fragment_factory = ColourFragment::new();
                    let builder = layer.get_process_builder(&vertex_factory,&fragment_factory)?;
                    let mut polygons = vertex_factory.make(builder,&centre,&radius,points,angle,depth,left,&group,wobble)?;
                    let draw = fragment_factory.make(builder)?;
                    let campaign = polygons.elements_mut();
                    add_colour(campaign,&patina,&draw)?;
                    campaign.close()?;
                    Ok(ShapeToAdd::Dynamic(Box::new(SolidPolygon::new(polygons,&gl))))
                },
                SimpleShapePatina::Hollow(_) => {
                    todo!()
                }
                SimpleShapePatina::Hotspot(hotspot) => {
                    let top_left = centre.merge_eoe(radius.clone(),SpaceBasePoint {
                        base: &|c,_r| *c,
                        normal: &|c,r| *c-*r,
                        tangent: &|c,r| *c-*r,
                        allotment: ()
                    });
                    let borrom_right = centre.merge_eoe(radius,SpaceBasePoint {
                        base: &|c,_r| *c,
                        normal: &|c,r| *c+*r,
                        tangent: &|c,r| *c+*r,
                        allotment: ()
                    });
                    let area = SpaceBaseArea::new(PartialSpaceBase::from_spacebase(top_left),PartialSpaceBase::from_spacebase(borrom_right)).expect("polygon hotspot");
                    Ok(ShapeToAdd::Hotspot(area,hotspot))
                },
                _ => {
                    Ok(ShapeToAdd::None)
                }
            }
        }
    }
}
