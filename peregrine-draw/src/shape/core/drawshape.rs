use peregrine_data::{Allotment, AllotmentDirection, AllotmentGroup, Colour, DataFilterBuilder, DirectColour, Flattenable, HoleySpaceBase, HoleySpaceBaseArea, HollowEdge, Patina, Plotter, SpaceBaseArea, ZMenu};
use super::directcolourdraw::DirectYielder;
use super::text::TextHandle;
use super::bitmap::BitmapHandle;
use super::super::layers::layer::{ Layer };
use super::texture::{CanvasTextureArea, TextureYielder};
use crate::shape::core::wigglegeometry::{WiggleYielder, make_wiggle};
use crate::shape::heraldry::heraldry::{HeraldryCanvas, HeraldryHandle, HeraldryScale};
use crate::shape::layers::drawing::DynamicShape;
use crate::shape::layers::patina::PatinaYielder;
use crate::shape::triangles::rectangles::{Rectangles };
use crate::shape::triangles::triangleskind::TrianglesKind;
use crate::webgl::{ ProcessStanzaAddable };
use crate::webgl::global::WebGlGlobal;
use super::super::layers::drawing::DrawingTools;
use crate::util::message::Message;
use crate::webgl::canvas::flatstore::FlatId;

#[derive(Clone,PartialEq,Eq,Hash)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum AllotmentProgramKind {
    Track,
    Overlay(i64),
    BaseLabel,
    SpaceLabel
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub(crate) enum SimpleShapePatina {
    Solid(Vec<DirectColour>),
    Hollow(Vec<DirectColour>),
    ZMenu(ZMenu,Vec<(String,Vec<String>)>)
}

fn simplify_colours(mut colours: Vec<Colour>) -> Result<Vec<DirectColour>,Message> {
    colours.drain(..).map(|colour| {
        match colour {
            Colour::Direct(d) => Ok(d),
            _ => Err(Message::CodeInvariantFailed(format!("attempt to simplify pattern to colour")))
        }
    }).collect::<Result<Vec<_>,_>>()
}

impl SimpleShapePatina {
    pub(crate) fn from_patina(patina: Patina) -> Result<SimpleShapePatina,Message> {
        Ok(match patina {
            Patina::Filled(colours,_) => { SimpleShapePatina::Solid(simplify_colours(colours)?) },
            Patina::Hollow(colours,_,_) => { SimpleShapePatina::Hollow(simplify_colours(colours)?) },
            Patina::ZMenu(zmenu,values) => { SimpleShapePatina::ZMenu(zmenu,values) }
        })
    }

    fn build(&self) -> DrawingShapePatina {
        match self {
            SimpleShapePatina::Solid(c) => DrawingShapePatina::Solid(DirectYielder::new(),c),
            SimpleShapePatina::Hollow(c) => DrawingShapePatina::Hollow(DirectYielder::new(),c),
            SimpleShapePatina::ZMenu(zmenu,values) => DrawingShapePatina::ZMenu(zmenu.clone(),values.clone())
        }
    }
}

enum DrawingShapePatina<'a> {
    Solid(DirectYielder,&'a [DirectColour]),
    Hollow(DirectYielder,&'a [DirectColour]),
    ZMenu(ZMenu,Vec<(String,Vec<String>)>)
}

enum PatinaTarget<'a> {
    Visual(&'a mut dyn PatinaYielder),
    HotSpot(ZMenu,Vec<(String,Vec<String>)>)
}

impl<'a> DrawingShapePatina<'a> {
    pub(crate) fn yielder_mut(&mut self) -> PatinaTarget {
        match self {
            DrawingShapePatina::Solid(dc,_) => PatinaTarget::Visual(dc),
            DrawingShapePatina::Hollow(dc,_) => PatinaTarget::Visual(dc),
            DrawingShapePatina::ZMenu(zmenu,values) => PatinaTarget::HotSpot(zmenu.clone(),values.clone())
        }
    }
}

pub(crate) enum GLShape {
    Text(HoleySpaceBase,Vec<TextHandle>,Vec<Allotment>,AllotmentProgramKind,i8),
    Image(HoleySpaceBase,Vec<BitmapHandle>,Vec<Allotment>,AllotmentProgramKind,i8),
    Heraldry(HoleySpaceBaseArea,Vec<HeraldryHandle>,Vec<Allotment>,AllotmentProgramKind,HeraldryCanvas,HeraldryScale,Option<HollowEdge<f64>>,i8),
    Wiggle((f64,f64),Vec<Option<f64>>,Plotter,Allotment,i8),
    SpaceBaseRect(HoleySpaceBaseArea,SimpleShapePatina,Vec<Allotment>,AllotmentProgramKind,i8),
}

pub enum AllotmentProgram {
    Track,
    Overlay(i64),
    BaseLabel(AllotmentDirection),
    SpaceLabel(AllotmentDirection)
}

impl AllotmentProgram {
    pub(super) fn kind(&self) -> AllotmentProgramKind {
        match self {
            AllotmentProgram::Track => AllotmentProgramKind::Track,
            AllotmentProgram::Overlay(p) => AllotmentProgramKind::Overlay(*p),
            AllotmentProgram::SpaceLabel(_) => AllotmentProgramKind::SpaceLabel,
            AllotmentProgram::BaseLabel(_) => AllotmentProgramKind::BaseLabel
        }        
    }
}

impl AllotmentProgram {
    pub(super) fn new(allotment: &AllotmentGroup) -> AllotmentProgram {
        match allotment {
            AllotmentGroup::Track => AllotmentProgram::Track,
            AllotmentGroup::Overlay(p) => AllotmentProgram::Overlay(*p),
            AllotmentGroup::SpaceLabel(x) => AllotmentProgram::SpaceLabel(x.clone()),
            AllotmentGroup::BaseLabel(x) => AllotmentProgram::BaseLabel(x.clone())
        }
    }
}

fn add_colour(addable: &mut dyn ProcessStanzaAddable, simple_shape_patina: &DrawingShapePatina) -> Result<(),Message> {
    let vertexes = match simple_shape_patina {
        DrawingShapePatina::Solid(_,_) => 4,
        DrawingShapePatina::Hollow(_,_) => 8,
        _ => 0
    };
    match simple_shape_patina {
        DrawingShapePatina::Solid(direct,colours) | DrawingShapePatina::Hollow(direct,colours) => {
            direct.draw()?.direct(addable,&colours,vertexes)?;
        },
        DrawingShapePatina::ZMenu(_,_) => {}
    }
    Ok(())
}

fn to_trianges_kind(program_kind: &AllotmentProgramKind) -> TrianglesKind {
    match program_kind {
        AllotmentProgramKind::Track => TrianglesKind::Track,
        AllotmentProgramKind::BaseLabel => TrianglesKind::Base,
        AllotmentProgramKind::SpaceLabel => TrianglesKind::Space,
        AllotmentProgramKind::Overlay(p) => TrianglesKind::Window(*p)
    }
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

fn draw_area_from_canvas(layer: &mut Layer, gl: &WebGlGlobal, kind: &TrianglesKind, area: &HoleySpaceBaseArea, allotments: &[Allotment], canvas: &FlatId, dims: &[CanvasTextureArea], free: bool, edge: &Option<HollowEdge<f64>>, priority: i8) -> Result<Box<dyn DynamicShape>,Message> {
    let mut geometry_yielder = kind.geometry_yielder(priority);
    let mut patina_yielder = TextureYielder::new(canvas,free);
    let left = layer.left();
    let mut rectangles = Rectangles::new_area(layer, &mut geometry_yielder, &mut patina_yielder,area,allotments,left,false,&kind,edge)?;
    let campaign = rectangles.elements_mut();
    patina_yielder.draw()?.add_rectangle(campaign,&canvas,&dims,gl.flat_store())?;
    campaign.close()?;
    Ok(Box::new(rectangles))
}

fn draw_points_from_canvas(layer: &mut Layer, gl: &WebGlGlobal, kind: &TrianglesKind, points: &HoleySpaceBase, x_sizes: Vec<f64>, y_sizes:Vec<f64>, allotments: &[Allotment], canvas: &FlatId, dims: &[CanvasTextureArea], free: bool, priority: i8) -> Result<Box<dyn DynamicShape>,Message> {
    let mut geometry_yielder = kind.geometry_yielder(priority);
    let mut patina_yielder = TextureYielder::new(canvas,free);
    let left = layer.left();
    let mut rectangles = Rectangles::new_sized(layer, &mut geometry_yielder, &mut patina_yielder,points,x_sizes,y_sizes,allotments,left,false,&kind)?;
    let campaign = rectangles.elements_mut();
    patina_yielder.draw()?.add_rectangle(campaign,&canvas,&dims,gl.flat_store())?;
    campaign.close()?;
    Ok(Box::new(rectangles))
}

fn draw_heraldry_canvas(layer: &mut Layer, gl: &WebGlGlobal, tools: &mut DrawingTools, kind: &TrianglesKind, area_a: &HoleySpaceBaseArea, handles: &[HeraldryHandle], allotments: &[Allotment], heraldry_canvas: &HeraldryCanvas, scale: &HeraldryScale, edge: &Option<HollowEdge<f64>>, priority: i8) -> Result<Option<Box<dyn DynamicShape>>,Message> {
    let heraldry = tools.heraldry();
    let mut dims = vec![];
    let mut filter_builder = DataFilterBuilder::new();
    for (i,handle) in handles.iter().enumerate() {
        let area = heraldry.get_texture_area(handle,&heraldry_canvas)?;
        if let Some(area) = area {
            dims.push(area);
            filter_builder.at(i);
        }
    }
    let mut filter = filter_builder.finish(handles.len());
    filter.set_size(area_a.len());
    if filter.count() == 0 { return Ok(None); }
    let canvas = heraldry.canvas_id(&heraldry_canvas).ok_or_else(|| Message::CodeInvariantFailed("no canvas id A".to_string()))?;
    Ok(Some(draw_area_from_canvas(layer,gl,kind,&area_a.filter(&filter),allotments,&canvas,&dims,scale.is_free(),edge,priority)?))
}

pub(crate) enum ShapeToAdd {
    Dynamic(Box<dyn DynamicShape>),
    ZMenu(SpaceBaseArea<f64>,Vec<Allotment>,ZMenu,Vec<(String,Vec<String>)>),
    None
}

pub(crate) fn add_shape_to_layer(layer: &mut Layer, gl: &WebGlGlobal, tools: &mut DrawingTools, shape: GLShape) -> Result<ShapeToAdd,Message> {
    match shape {
        GLShape::Wiggle((start,end),yy,Plotter(height,colour),allotment,prio) => {
            let mut geometry_yielder = WiggleYielder::new(prio);
            let mut patina_yielder = DirectYielder::new(); // XXX spot
            let left = layer.left();
            let mut array = make_wiggle(layer,&mut geometry_yielder,&mut patina_yielder,start,end,yy,height,&allotment,left)?;
            patina_yielder.draw()?.direct(&mut array,&[colour],1)?;
            array.close()?;
            Ok(ShapeToAdd::None)
        },
        GLShape::Text(points,handles,allotments,program_kind,prio) => {
            let kind = to_trianges_kind(&program_kind);
            // TODO factor
            let text = tools.text();
            let dims = handles.iter()
                .map(|handle| text.manager().get_texture_areas(handle))
                .collect::<Result<Vec<_>,_>>()?;
            let (x_sizes,y_sizes) = dims_to_sizes(&dims);
            let canvas = text.manager().canvas_id().ok_or_else(|| Message::CodeInvariantFailed("no canvas id A".to_string()))?;
            let rectangles = draw_points_from_canvas(layer,gl,&kind,&points,x_sizes,y_sizes,&allotments,&canvas,&dims,false,prio)?;
            Ok(ShapeToAdd::Dynamic(rectangles))
        },
        GLShape::Image(points,handles,allotments,program_kind,prio) => {
            let kind = to_trianges_kind(&program_kind);
            // TODO factor
            let bitmap = tools.bitmap();
            let dims = handles.iter()
                .map(|handle| bitmap.manager().get_texture_areas(handle))
                .collect::<Result<Vec<_>,_>>()?;
            let (x_sizes,y_sizes) = dims_to_sizes(&dims);
            let canvas = bitmap.manager().canvas_id().ok_or_else(|| Message::CodeInvariantFailed("no canvas id A".to_string()))?;
            let rectangles = draw_points_from_canvas(layer,gl,&kind,&points,x_sizes,y_sizes,&allotments,&canvas,&dims,false,prio)?;
            Ok(ShapeToAdd::Dynamic(rectangles))
        },
        GLShape::Heraldry(area,handles,allotments,program_kind,heraldry_canvas,scale,edge,prio) => {
            let kind = to_trianges_kind(&program_kind);
            let rectangles = draw_heraldry_canvas(layer,gl,tools,&kind,&area,&handles,&allotments,&heraldry_canvas,&scale,&edge,prio)?;
            if let Some(rectangles) = rectangles {
                Ok(ShapeToAdd::Dynamic(rectangles))
            } else {
                Ok(ShapeToAdd::None)
            }
        },
        GLShape::SpaceBaseRect(area,simple_shape_patina,allotments,allotment_kind,prio) => {
            let mut drawing_shape_patina = simple_shape_patina.build();
            let kind = to_trianges_kind(&allotment_kind);
            let mut geometry_yielder = kind.geometry_yielder(prio);
            let left = layer.left();
            match drawing_shape_patina.yielder_mut() {
                PatinaTarget::Visual(patina_yielder) => {
                    let hollow = match simple_shape_patina { SimpleShapePatina::Hollow(_) => true, _ => false };
                    let mut rectangles = Rectangles::new_area(layer,&mut geometry_yielder,patina_yielder,&area,&allotments,left,hollow,&kind,&None)?;
                    let campaign = rectangles.elements_mut();
                    add_colour(campaign,&drawing_shape_patina)?;
                    campaign.close()?;
                    Ok(ShapeToAdd::Dynamic(Box::new(rectangles)))
                },
                PatinaTarget::HotSpot(zmenu,values) => {
                    let (real_area,_subs) = area.extract();
                    Ok(ShapeToAdd::ZMenu(real_area,allotments,zmenu,values))
                }
            }
        }
    }
}
