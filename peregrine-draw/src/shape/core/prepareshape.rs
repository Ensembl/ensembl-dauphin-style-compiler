use peregrine_data::{Allotment, AllotmentRequest, Colour, EachOrEvery, HoleySpaceBaseArea, HollowEdge, Patina, Shape, ShapeDemerge, ZMenu};
use super::super::layers::layer::{ Layer };
use super::super::layers::drawing::DrawingTools;
use crate::shape::core::drawshape::{SimpleShapePatina};
use crate::shape::heraldry::heraldry::{Heraldry, HeraldryCanvasesUsed, HeraldryScale};
use crate::shape::triangles::drawgroup::{DrawGroup, DrawGroupVariety};
use crate::util::message::Message;
use super::drawshape::{ GLShape };
use std::hash::Hash;
use std::sync::Arc;

// XXX not a new one for each!
fn allotments(allotments: &[AllotmentRequest]) -> Result<Vec<Allotment>,Message> {
    allotments.iter().map(|handle| {
        handle.allotment().map(|a| a.clone())
    }).collect::<Result<Vec<_>,_>>().map_err(|e| Message::DataError(e))
}

#[derive(Clone,PartialEq,Eq,Hash)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub(crate) enum ShapeCategory {
    Solid(),
    Heraldry(HeraldryCanvasesUsed,HeraldryScale)
}

enum PatinaExtract {
    Visual(Arc<EachOrEvery<Colour>>,Option<u32>),
    ZMenu(ZMenu,Arc<Vec<(String,EachOrEvery<String>)>>)
}

fn extract_patina(patina: &Patina) -> PatinaExtract {
    match &patina {
        Patina::Filled(c) => PatinaExtract::Visual(c.clone(),None),
        Patina::Hollow(c,w) => PatinaExtract::Visual(c.clone(),Some(*w)),
        Patina::ZMenu(zmenu,values) => PatinaExtract::ZMenu(zmenu.clone(),values.clone())
    }
}

fn split_spacebaserect(tools: &mut DrawingTools, area: HoleySpaceBaseArea, patina: Patina, allotment: Vec<AllotmentRequest>, draw_group: &DrawGroup) -> Result<Vec<GLShape>,Message> {
    let allotment = allotments(&allotment)?;
    let mut out = vec![];
    match extract_patina(&patina) {
        PatinaExtract::Visual(colours,width) => {
            let mut demerge_colour = colours.demerge(|colour| {
                if let Some(heraldry) = colour_to_heraldry(colour,width.is_some()) {
                    ShapeCategory::Heraldry(heraldry.canvases_used(),heraldry.scale())                                
                } else {
                    ShapeCategory::Solid()
                }
            });
            for (pkind,filter) in &mut demerge_colour {
                filter.set_size(area.len());
                match pkind {
                    ShapeCategory::Solid() => {
                        out.push(GLShape::SpaceBaseRect(area.filter(filter),SimpleShapePatina::from_patina(patina.filter(filter))?,filter.filter(&allotment),draw_group.clone()));
                    },
                    ShapeCategory::Heraldry(HeraldryCanvasesUsed::Solid(heraldry_canvas),scale) => {
                        let heraldry_tool = tools.heraldry();
                        let heraldry = make_heraldry(patina.filter(filter))?;
                        let handles = heraldry.map_into(|x| heraldry_tool.add(x));
                        let area = area.filter(filter);
                        let allotment = filter.filter(&allotment);
                        out.push(GLShape::Heraldry(area,Arc::new(handles),allotment,draw_group.clone(),heraldry_canvas.clone(),scale.clone(),None));
                    },
                    ShapeCategory::Heraldry(HeraldryCanvasesUsed::Hollow(heraldry_canvas_h,heraldry_canvas_v),scale) => {
                        let width = width.unwrap_or(0) as f64;
                        let heraldry_tool = tools.heraldry();
                        let heraldry = make_heraldry(patina.filter(filter))?;
                        let handles = Arc::new(heraldry.map_into(|x| heraldry_tool.add(x)));
                        let area = area.filter(filter);
                        let allotment = filter.filter(allotment.as_ref());
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
            out.push(GLShape::SpaceBaseRect(area,SimpleShapePatina::ZMenu(zmenu,values),allotment,draw_group.clone()));
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
            Shape::Wiggle(range,y,plotter,allotment,_) => {
                let allotment = allotments(&[allotment])?;
                out.push(GLShape::Wiggle(range,y,plotter,allotment[0].clone()));
            },
            Shape::Text(spacebase,pen,texts,allotment,_) => {
                let allotment = allotments(&allotment)?;
                let drawing_text = tools.text();
                let colours_iter = pen.colours().iter().cycle();
                let background = pen.background();
                let handles : Vec<_> = texts.iter().zip(colours_iter).map(|(text,colour)| drawing_text.add_text(&pen,text,colour,background)).collect();
                out.push(GLShape::Text(spacebase,handles,allotment,draw_group));
            },
            Shape::Image(spacebase,images,allotment,_) => {
                let allotment = allotments(&allotment)?;
                let drawing_bitmap = tools.bitmap();
                let handles = images.iter().map(|asset| drawing_bitmap.add_bitmap(asset)).collect::<Result<Vec<_>,_>>()?;
                out.push(GLShape::Image(spacebase,handles,allotment,draw_group));
            },
            Shape::SpaceBaseRect(area,patina,allotment,_) => {
                out.append(&mut split_spacebaserect(tools,area,patina,allotment,&draw_group)?);
            }
        }    
    }
    Ok(out)
}
