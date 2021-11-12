use std::sync::Arc;

use peregrine_data::{Allotment, Colour, DataFilterBuilder, DirectColour, DrawnType, EachOrEvery, Flattenable, HoleySpaceBase, HoleySpaceBaseArea, HollowEdge, Patina, Plotter, SpaceBaseArea, ZMenu};
use super::directcolourdraw::DirectYielder;
use super::spotcolourdraw::SpotColourYielder;
use super::text::TextHandle;
use super::bitmap::BitmapHandle;
use super::super::layers::layer::{ Layer };
use super::texture::{CanvasTextureArea, TextureYielder};
use crate::shape::core::wigglegeometry::{make_wiggle};
use crate::shape::heraldry::heraldry::{HeraldryCanvas, HeraldryHandle, HeraldryScale};
use crate::shape::layers::drawing::DynamicShape;
use crate::shape::layers::geometry::{GeometryYielder, GeometryProcessName };
use crate::shape::layers::patina::PatinaYielder;
use crate::shape::triangles::rectangles::{Rectangles };
use crate::shape::triangles::drawgroup::DrawGroup;
use crate::shape::util::iterators::eoe_throw;
use crate::webgl::{ ProcessStanzaAddable };
use crate::webgl::global::WebGlGlobal;
use super::super::layers::drawing::DrawingTools;
use crate::util::message::Message;
use crate::webgl::canvas::flatstore::FlatId;

#[cfg_attr(debug_assertions,derive(Debug))]
pub(crate) enum LineColour {
    Direct(EachOrEvery<DirectColour>),
    Spot(DirectColour),
//    Heraldry(EachOrEvery<HeraldryHandle>)
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub(crate) enum SimpleShapePatina {
    Solid(EachOrEvery<DirectColour>),
    Hollow(EachOrEvery<DirectColour>),
    SolidSpot(DirectColour),
    HollowSpot(DirectColour),
    ZMenu(ZMenu,Vec<(String,EachOrEvery<String>)>)
}

pub(super) fn simplify_colours(colours: &EachOrEvery<Colour>) -> Result<EachOrEvery<DirectColour>,Message> {
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
            Patina::ZMenu(zmenu,values) => { SimpleShapePatina::ZMenu(zmenu.clone(),values.clone()) }
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
            Patina::ZMenu(zmenu,values) => { SimpleShapePatina::ZMenu(zmenu.clone(),values.clone()) }
        })
    }

    fn build(&self) -> DrawingShapePatina {
        match self {
            SimpleShapePatina::Solid(c) => DrawingShapePatina::Solid(DirectYielder::new(),c.clone()),
            SimpleShapePatina::Hollow(c) => DrawingShapePatina::Hollow(DirectYielder::new(),c.clone()),
            SimpleShapePatina::SolidSpot(c) => DrawingShapePatina::SolidSpot(SpotColourYielder::new(c)),
            SimpleShapePatina::HollowSpot(c) => DrawingShapePatina::HollowSpot(SpotColourYielder::new(c)),
            SimpleShapePatina::ZMenu(zmenu,values) => DrawingShapePatina::ZMenu(zmenu.clone(),values.clone())
        }
    }
}

enum DrawingShapePatina {
    Solid(DirectYielder,EachOrEvery<DirectColour>),
    Hollow(DirectYielder,EachOrEvery<DirectColour>),
    SolidSpot(SpotColourYielder),
    HollowSpot(SpotColourYielder),
    ZMenu(ZMenu,Vec<(String,EachOrEvery<String>)>)
}

enum PatinaTarget<'a> {
    Visual(&'a mut dyn PatinaYielder),
    HotSpot(ZMenu,Vec<(String,EachOrEvery<String>)>)
}

impl DrawingShapePatina {
    pub(crate) fn yielder_mut(&mut self) -> PatinaTarget {
        match self {
            DrawingShapePatina::Solid(dc,_) => PatinaTarget::Visual(dc),
            DrawingShapePatina::Hollow(dc,_) => PatinaTarget::Visual(dc),
            DrawingShapePatina::SolidSpot(dc) => PatinaTarget::Visual(dc),
            DrawingShapePatina::HollowSpot(dc) => PatinaTarget::Visual(dc),
            DrawingShapePatina::ZMenu(zmenu,values) => PatinaTarget::HotSpot(zmenu.clone(),values.clone())
        }
    }
}

pub(crate) enum GLShape {
    Text(HoleySpaceBase,Vec<TextHandle>,EachOrEvery<Allotment>,DrawGroup),
    Image(HoleySpaceBase,Vec<BitmapHandle>,EachOrEvery<Allotment>,DrawGroup),
    Heraldry(HoleySpaceBaseArea,EachOrEvery<HeraldryHandle>,EachOrEvery<Allotment>,DrawGroup,HeraldryCanvas,HeraldryScale,Option<HollowEdge<f64>>),
    Wiggle((f64,f64),Arc<Vec<Option<f64>>>,Plotter,Allotment),
    Rectangle(HoleySpaceBaseArea,SimpleShapePatina,EachOrEvery<Allotment>,DrawGroup),
    Line(HoleySpaceBaseArea,LineColour,u32,EachOrEvery<Allotment>,DrawGroup),
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

fn dims_to_sizes(areas: &[CanvasTextureArea]) -> (Vec<f64>,Vec<f64>) {
    let mut x_sizes = vec![];
    let mut y_sizes = vec![];
    for dim in areas {
        let size = dim.size();
        x_sizes.push(size.0 as f64);
        y_sizes.push(size.1 as f64);
    }
    (x_sizes,y_sizes)
}

fn draw_area_from_canvas(layer: &mut Layer, gl: &WebGlGlobal, draw_group: &DrawGroup, area: &HoleySpaceBaseArea, allotments: &EachOrEvery<Allotment>, canvas: &FlatId, dims: &[CanvasTextureArea], free: bool, edge: &Option<HollowEdge<f64>>) -> Result<Box<dyn DynamicShape>,Message> {
    let mut geometry_yielder = draw_group.geometry_yielder();
    let mut patina_yielder = TextureYielder::new(canvas,free);
    let left = layer.left();
    let mut rectangles = Rectangles::new_area(layer, &mut geometry_yielder, &mut patina_yielder,area,allotments,left,false,&draw_group,edge)?;
    let campaign = rectangles.elements_mut();
    patina_yielder.draw()?.add_rectangle(campaign,&canvas,&dims,gl.flat_store())?;
    campaign.close()?;
    Ok(Box::new(rectangles))
}

fn draw_points_from_canvas(layer: &mut Layer, gl: &WebGlGlobal, draw_group: &DrawGroup, points: &HoleySpaceBase, x_sizes: Vec<f64>, y_sizes:Vec<f64>, allotments: &EachOrEvery<Allotment>, canvas: &FlatId, dims: &[CanvasTextureArea], free: bool) -> Result<Box<dyn DynamicShape>,Message> {
    let mut geometry_yielder = draw_group.geometry_yielder();
    let mut patina_yielder = TextureYielder::new(canvas,free);
    let left = layer.left();
    let mut rectangles = Rectangles::new_sized(layer, &mut geometry_yielder, &mut patina_yielder,points,x_sizes,y_sizes,allotments,left,false,&draw_group)?;
    let campaign = rectangles.elements_mut();
    patina_yielder.draw()?.add_rectangle(campaign,&canvas,&dims,gl.flat_store())?;
    campaign.close()?;
    Ok(Box::new(rectangles))
}

fn draw_heraldry_canvas(layer: &mut Layer, gl: &WebGlGlobal, tools: &mut DrawingTools, kind: &DrawGroup, area_a: &HoleySpaceBaseArea, handles: &EachOrEvery<HeraldryHandle>, allotments: &EachOrEvery<Allotment>, heraldry_canvas: &HeraldryCanvas, scale: &HeraldryScale, edge: &Option<HollowEdge<f64>>, count: usize) -> Result<Option<Box<dyn DynamicShape>>,Message> {
    let heraldry = tools.heraldry();
    let mut dims = vec![];
    let mut filter_builder = DataFilterBuilder::new();
    for (i,handle) in eoe_throw("heraldry",handles.iter(count))?.enumerate() {
        let area = heraldry.get_texture_area(handle,&heraldry_canvas)?;
        if let Some(area) = area {
            dims.push(area);
            filter_builder.at(i);
        }
    }
    let mut filter = filter_builder.finish(count);
    filter.set_size(area_a.len());
    if filter.count() == 0 { return Ok(None); }
    let canvas = heraldry.canvas_id(&heraldry_canvas).ok_or_else(|| Message::CodeInvariantFailed("no canvas id A".to_string()))?;
    Ok(Some(draw_area_from_canvas(layer,gl,kind,&area_a.filter(&filter),allotments,&canvas,&dims,scale.is_free(),edge)?))
}

pub(crate) enum ShapeToAdd {
    Dynamic(Box<dyn DynamicShape>),
    ZMenu(SpaceBaseArea<f64>,EachOrEvery<Allotment>,ZMenu,Vec<(String,EachOrEvery<String>)>),
    None
}

pub(crate) fn add_shape_to_layer(layer: &mut Layer, gl: &WebGlGlobal, tools: &mut DrawingTools, shape: GLShape) -> Result<ShapeToAdd,Message> {
    match shape {
        GLShape::Wiggle((start,end),yy,Plotter(height,colour),allotment) => {
            let mut geometry_yielder = GeometryYielder::new(GeometryProcessName::Wiggle,allotment.depth());
            let mut patina_yielder = DirectYielder::new();
            let left = layer.left();
            let (mut array,count) = make_wiggle(layer,&mut geometry_yielder,&mut patina_yielder,start,end,&yy,height,&allotment,left,allotment.depth())?;
            patina_yielder.draw()?.direct(&mut array, &EachOrEvery::Every(colour),1,count)?;
            array.close()?;
            Ok(ShapeToAdd::None)
        },
        GLShape::Text(points,handles,allotments,draw_group) => {
            // TODO factor
            let text = tools.text();
            let dims = handles.iter()
                .map(|handle| text.manager().get_texture_areas(handle))
                .collect::<Result<Vec<_>,_>>()?;
            if dims.len() == 0 { return Ok(ShapeToAdd::None); }
            let (x_sizes,y_sizes) = dims_to_sizes(&dims);
            let canvas = text.manager().canvas_id().ok_or_else(|| Message::CodeInvariantFailed("no canvas id A".to_string()))?;
            let rectangles = draw_points_from_canvas(layer,gl,&draw_group,&points,x_sizes,y_sizes,&allotments,&canvas,&dims,false)?;
            Ok(ShapeToAdd::Dynamic(rectangles))
        },
        GLShape::Image(points,handles,allotments,kind) => {
            // TODO factor
            let bitmap = tools.bitmap();
            let dims = handles.iter()
                .map(|handle| bitmap.manager().get_texture_areas(handle))
                .collect::<Result<Vec<_>,_>>()?;
            if dims.len() == 0 { return Ok(ShapeToAdd::None); }
                let (x_sizes,y_sizes) = dims_to_sizes(&dims);
            let canvas = bitmap.manager().canvas_id().ok_or_else(|| Message::CodeInvariantFailed("no canvas id A".to_string()))?;
            let rectangles = draw_points_from_canvas(layer,gl,&kind,&points,x_sizes,y_sizes,&allotments,&canvas,&dims,false)?;
            Ok(ShapeToAdd::Dynamic(rectangles))
        },
        GLShape::Heraldry(area,handles,allotments,kind,heraldry_canvas,scale,edge) => {
            let rectangles = draw_heraldry_canvas(layer,gl,tools,&kind,&area,&handles,&allotments,&heraldry_canvas,&scale,&edge,area.len())?;
            if let Some(rectangles) = rectangles {
                Ok(ShapeToAdd::Dynamic(rectangles))
            } else {
                Ok(ShapeToAdd::None)
            }
        },
        GLShape::Rectangle(area,simple_shape_patina,allotments,draw_group) => {
            let mut drawing_shape_patina = simple_shape_patina.build();
            let mut geometry_yielder = draw_group.geometry_yielder();
            let left = layer.left();
            match drawing_shape_patina.yielder_mut() {
                PatinaTarget::Visual(patina_yielder) => {
                    let hollow = match simple_shape_patina { SimpleShapePatina::Hollow(_) | SimpleShapePatina::HollowSpot(_) => true, _ => false };
                    let mut rectangles = Rectangles::new_area(layer,&mut geometry_yielder,patina_yielder,&area,&allotments,left,hollow,&draw_group,&None)?;
                    let campaign = rectangles.elements_mut();
                    add_colour(campaign,&drawing_shape_patina,area.len())?;
                    campaign.close()?;
                    Ok(ShapeToAdd::Dynamic(Box::new(rectangles)))
                },
                PatinaTarget::HotSpot(zmenu,values) => {
                    let (real_area,_subs) = area.extract();
                    Ok(ShapeToAdd::ZMenu(real_area,allotments,zmenu,values))
                }
            }
        },
        GLShape::Line(line,colour,width,allotments,draw_group) => {
            use web_sys::console;
            #[cfg(debug_assertions)]
            console::log_1(&format!("line: line={:?}",line).into());
            Ok(ShapeToAdd::None)
        }
    }
}
