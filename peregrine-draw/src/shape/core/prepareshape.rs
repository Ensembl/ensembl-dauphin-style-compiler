use peregrine_data::{Allotment, AllotmentRequest, Colour, EachOrEvery, HollowEdge, Patina, RectangleShape, Shape, ShapeDemerge, ZMenu};
use super::super::layers::layer::{ Layer };
use super::super::layers::drawing::DrawingTools;
use crate::shape::core::drawshape::{SimpleShapePatina};
use crate::shape::heraldry::heraldry::{Heraldry, HeraldryCanvasesUsed, HeraldryScale};
use crate::shape::triangles::drawgroup::{DrawGroup, DrawGroupVariety};
use crate::util::message::Message;
use super::drawshape::{ GLShape };
use std::hash::Hash;

fn get_allotment(handle: &AllotmentRequest) -> Result<Allotment,Message> {
    handle.allotment().map_err(|e| Message::DataError(e))
}

fn allotments(allotments: &EachOrEvery<AllotmentRequest>) -> Result<EachOrEvery<Allotment>,Message> {
    allotments.map_results(|handle|{
        handle.allotment()
    }).map_err(|e| Message::DataError(e))
}

#[derive(Clone,PartialEq,Eq,Hash)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub(crate) enum ShapeCategory {
    Solid(),
    Heraldry(HeraldryCanvasesUsed,HeraldryScale)
}

enum PatinaExtract {
    Visual(EachOrEvery<Colour>,Option<u32>),
    ZMenu(ZMenu,Vec<(String,EachOrEvery<String>)>)
}

fn extract_patina(patina: &Patina) -> PatinaExtract {
    match &patina {
        Patina::Filled(c) => PatinaExtract::Visual(c.clone(),None),
        Patina::Hollow(c,w) => PatinaExtract::Visual(c.clone(),Some(*w)),
        Patina::ZMenu(zmenu,values) => PatinaExtract::ZMenu(zmenu.clone(),values.clone())
    }
}

fn split_spacebaserect(tools: &mut DrawingTools, shape: &RectangleShape, draw_group: &DrawGroup) -> Result<Vec<GLShape>,Message> {
    let allotment = allotments(shape.allotments())?;
    let mut out = vec![];
    match extract_patina(&shape.patina()) {
        PatinaExtract::Visual(colours,width) => {
            let mut demerge_colour = colours.demerge(|colour| {
                if let Some(heraldry) = colour_to_heraldry(colour,width.is_some()) {
                    ShapeCategory::Heraldry(heraldry.canvases_used(),heraldry.scale())                                
                } else {
                    ShapeCategory::Solid()
                }
            });
            for (pkind,filter) in &mut demerge_colour {
                filter.set_size(shape.len());
                match pkind {
                    ShapeCategory::Solid() => {
                        out.push(GLShape::SpaceBaseRect(shape.holey_area().filter(filter),SimpleShapePatina::from_patina(shape.patina().filter(filter))?,allotment.filter(&filter),draw_group.clone()));
                    },
                    ShapeCategory::Heraldry(HeraldryCanvasesUsed::Solid(heraldry_canvas),scale) => {
                        let heraldry_tool = tools.heraldry();
                        let heraldry = make_heraldry(shape.patina().filter(filter))?;
                        let handles = heraldry.map(|x| heraldry_tool.add(x.clone()));
                        let area = shape.holey_area().filter(filter);
                        let allotment = allotment.filter(&filter);
                        out.push(GLShape::Heraldry(area,handles,allotment,draw_group.clone(),heraldry_canvas.clone(),scale.clone(),None));
                    },
                    ShapeCategory::Heraldry(HeraldryCanvasesUsed::Hollow(heraldry_canvas_h,heraldry_canvas_v),scale) => {
                        let width = width.unwrap_or(0) as f64;
                        let heraldry_tool = tools.heraldry();
                        let heraldry = make_heraldry(shape.patina().filter(filter))?;
                        let handles = heraldry.map(|x| heraldry_tool.add(x.clone()));
                        let area = shape.holey_area().filter(filter);
                        let allotment = allotment.filter(&filter);
                        // XXX too much cloning, at least Arc them
                        out.push(GLShape::Heraldry(area.clone(),handles.clone(),allotment.clone(),draw_group.clone(),heraldry_canvas_v.clone(),scale.clone(),Some(HollowEdge::Left(width))));
                        out.push(GLShape::Heraldry(area.clone(),handles.clone(),allotment.clone(),draw_group.clone(),heraldry_canvas_v.clone(),scale.clone(),Some(HollowEdge::Right(width))));
                        out.push(GLShape::Heraldry(area.clone(),handles.clone(),allotment.clone(),draw_group.clone(),heraldry_canvas_h.clone(),scale.clone(),Some(HollowEdge::Top(width))));
                        out.push(GLShape::Heraldry(area.clone(),handles,allotment,draw_group.clone(),heraldry_canvas_h.clone(),scale.clone(),Some(HollowEdge::Bottom(width))));
                    }
                }
            }        
        },
        PatinaExtract::ZMenu(zmenu,values) => {
            out.push(GLShape::SpaceBaseRect(shape.holey_area().clone(),SimpleShapePatina::ZMenu(zmenu,values),allotment,draw_group.clone()));
        }
    }
    Ok(out)
}

fn colour_to_heraldry(colour: &Colour, hollow: bool) -> Option<Heraldry> {
    match colour {
        Colour::Stripe(a,b,c,_prop) => {
            Some(Heraldry::Stripe(a.clone(),b.clone(),50,*c))
        },
        Colour::Bar(a,b,c,prop) => {
            if hollow {
                Some(Heraldry::new_dots(a,b,(prop*100.) as u32,*c,false))
            } else {
                Some(Heraldry::new_bar(a,b,(prop*100.) as u32,*c,false))
            }
        },
        _ => None
    }
}

fn make_heraldry(patina: Patina) -> Result<EachOrEvery<Heraldry>,Message> {
    let (colours,hollow) = match patina {
        Patina::Filled(c) => (c,false),
        Patina::Hollow(c,_) => (c,true),
        _ => Err(Message::CodeInvariantFailed(format!("heraldry attempted on non filled/hollow")))?
    };
    colours.map_results(|colour| {
        colour_to_heraldry(colour,hollow)
            .ok_or_else(|| Message::CodeInvariantFailed(format!("heraldry attempted on non-heraldic colour")))
    })
}

pub struct GLCategoriser();

impl ShapeDemerge for GLCategoriser {
    type X = DrawGroup;

    fn categorise(&self, allotment: &AllotmentRequest) -> Self::X {
        DrawGroup::new(&allotment.coord_system(),allotment.depth(),&DrawGroupVariety::Visual)
    }
}

pub(crate) fn prepare_shape_in_layer(_layer: &mut Layer, tools: &mut DrawingTools, shape: Shape) -> Result<Vec<GLShape>,Message> {
    let mut out = vec![];
    for (draw_group,shape) in shape.demerge(&GLCategoriser()) {
        match shape {
            Shape::Wiggle(shape) => {
                out.push(GLShape::Wiggle(shape.range(),shape.values(),shape.plotter().clone(),get_allotment(shape.allotment())?));
            },
            Shape::Text(shape) => {
                let allotment = allotments(&shape.allotments())?;
                let drawing_text = tools.text();
                let colours_iter = shape.pen().colours().iter().cycle();
                let background = shape.pen().background();
                let texts = shape.iter_texts().collect::<Vec<_>>();
                let handles : Vec<_> = texts.iter().zip(colours_iter).map(|(text,colour)| drawing_text.add_text(&shape.pen(),text,colour,background)).collect();
                out.push(GLShape::Text(shape.holey_position().clone(),handles,allotment,draw_group));
            },
            Shape::Image(shape) => {
                let allotment = allotments(shape.allotments())?;
                let drawing_bitmap = tools.bitmap();
                let names = shape.iter_names().collect::<Vec<_>>();
                let handles = names.iter().map(|asset| drawing_bitmap.add_bitmap(asset)).collect::<Result<Vec<_>,_>>()?;
                out.push(GLShape::Image(shape.holey_position().clone(),handles,allotment,draw_group));
            },
            Shape::SpaceBaseRect(shape) => {
                out.append(&mut split_spacebaserect(tools,&shape,&draw_group)?);
            }
        }    
    }
    Ok(out)
}
