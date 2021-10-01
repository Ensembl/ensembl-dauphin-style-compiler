use peregrine_data::{Allotment, AllotmentRequest, Colour, DataFilter, HoleySpaceBaseArea, HollowEdge, Patina, Shape, ZMenu};
use super::super::layers::layer::{ Layer };
use super::super::layers::drawing::DrawingTools;
use crate::shape::core::drawshape::{SimpleShapePatina};
use crate::shape::heraldry::heraldry::{Heraldry, HeraldryCanvasesUsed, HeraldryScale};
use crate::shape::triangles::drawgroup::DrawGroup;
use crate::util::message::Message;
use super::drawshape::{ GLShape };
use std::hash::Hash;

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

enum PatinaExtract<'a> {
    Visual(&'a [Colour],Option<u32>),
    ZMenu(ZMenu,Vec<(String,Vec<String>)>)
}

fn extract_patina<'a>(patina: &'a Patina) -> PatinaExtract<'a> {
    match &patina {
        Patina::Filled(c) => PatinaExtract::Visual(c,None),
        Patina::Hollow(c,w) => PatinaExtract::Visual(c,Some(*w)),
        Patina::ZMenu(zmenu,values) => PatinaExtract::ZMenu(zmenu.clone(),values.clone())
    }
}

fn split_spacebaserect(tools: &mut DrawingTools, area: HoleySpaceBaseArea, patina: Patina, allotment: Vec<AllotmentRequest>, draw_group: &DrawGroup) -> Result<Vec<GLShape>,Message> {
    let allotment = allotments(&allotment)?;
    let mut out = vec![];
    match extract_patina(&patina) {
        PatinaExtract::Visual(colours,width) => {
            let mut demerge_colour = DataFilter::demerge(&colours,|colour| {
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
                        let mut heraldry = make_heraldry(patina.filter(filter))?;
                        let handles = heraldry.drain(..).map(|x| heraldry_tool.add(x)).collect::<Vec<_>>();
                        let area = area.filter(filter);
                        let allotment = filter.filter(&allotment);
                        out.push(GLShape::Heraldry(area,handles,allotment,draw_group.clone(),heraldry_canvas.clone(),scale.clone(),None));
                    },
                    ShapeCategory::Heraldry(HeraldryCanvasesUsed::Hollow(heraldry_canvas_h,heraldry_canvas_v),scale) => {
                        let width = width.unwrap_or(0) as f64;
                        let heraldry_tool = tools.heraldry();
                        let mut heraldry = make_heraldry(patina.filter(filter))?;
                        let handles = heraldry.drain(..).map(|x| heraldry_tool.add(x)).collect::<Vec<_>>();
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
        Colour::Stripe(a,b,c,prop) => {
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

fn make_heraldry(patina: Patina) -> Result<Vec<Heraldry>,Message> {
    let (colours,hollow) = match patina {
        Patina::Filled(c) => (c,false),
        Patina::Hollow(c,_) => (c,true),
        _ => Err(Message::CodeInvariantFailed(format!("heraldry attempted on non filled/hollow")))?
    };
    let mut handles = vec![];
    for colour in &colours {
        let heraldry = colour_to_heraldry(colour,hollow)
            .ok_or_else(|| Message::CodeInvariantFailed(format!("heraldry attempted on non-heraldic colour")))?;
        handles.push(heraldry);
    }
    Ok(handles)
}

fn split_on_draw_group(shape: Shape) -> Vec<(DrawGroup,Shape)> {
    shape.demerge_by_allotment(|allotment| {
        DrawGroup::new(&allotment.coord_system(),allotment.depth())
    })
}

pub(crate) fn prepare_shape_in_layer(_layer: &mut Layer, tools: &mut DrawingTools, shape: Shape) -> Result<Vec<GLShape>,Message> {
    let mut out = vec![];
    for (draw_group,shape) in split_on_draw_group(shape) {
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
