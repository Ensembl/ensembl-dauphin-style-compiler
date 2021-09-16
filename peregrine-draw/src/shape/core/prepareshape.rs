use peregrine_data::{Allotment, AllotmentRequest, Allotter, Colour, DataFilter, HoleySpaceBaseArea, HollowEdge, Patina, Plotter, Shape, SpaceBaseArea, ZMenu};
use super::super::layers::layer::{ Layer };
use super::super::layers::drawing::DrawingTools;
use crate::shape::core::drawshape::SimpleShapePatina;
use crate::shape::heraldry::heraldry::{Heraldry, HeraldryCanvasesUsed, HeraldryScale};
use crate::util::message::Message;
use super::drawshape::{ GLShape, AllotmentProgram };


// XXX not a new one for each!
fn allotments(allotter: &Allotter, allotments: &[AllotmentRequest]) -> Result<Vec<Allotment>,Message> {
    allotments.iter().map(|handle| {
        allotter.get(handle).map(|a| a.clone())
    }).collect::<Result<Vec<_>,_>>().map_err(|e| Message::DataError(e))
}

#[derive(Clone,PartialEq,Eq,Hash)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub(crate) enum ShapeCategory {
    Solid(i8),
    Heraldry(HeraldryCanvasesUsed,HeraldryScale,i8)
}

enum PatinaExtract<'a> {
    Visual(&'a [Colour],Option<u32>,i8),
    ZMenu(ZMenu,Vec<(String,Vec<String>)>)
}

fn extract_patina<'a>(patina: &'a Patina) -> PatinaExtract<'a> {
    match &patina {
        Patina::Filled(c,prio) => PatinaExtract::Visual(c,None,*prio),
        Patina::Hollow(c,w,prio) => PatinaExtract::Visual(c,Some(*w),*prio),
        Patina::ZMenu(zmenu,values) => PatinaExtract::ZMenu(zmenu.clone(),values.clone())
    }
}

fn split_spacebaserect(tools: &mut DrawingTools, allotter: &Allotter, area: HoleySpaceBaseArea, patina:Patina, allotment: Vec<AllotmentRequest>) -> Result<Vec<GLShape>,Message> {
    let allotment = allotments(allotter,&allotment)?;
    let mut demerge = DataFilter::demerge(&allotment,|allotment| {
        AllotmentProgram::new(&allotment.position().kind()).kind()
    });
    let mut mid = vec![];
    for (kind,filter) in &mut demerge {
        filter.set_size(area.len());
        mid.push((area.filter(filter),patina.filter(filter),filter.filter(&allotment),kind.clone()));
    }
    let mut out = vec![];
    for (area,patina,allotment,kind) in mid {
        match extract_patina(&patina) {
            PatinaExtract::Visual(colours,width,prio) => {
                let mut demerge_colour = DataFilter::demerge(&colours,|colour| {
                    if let Some(heraldry) = colour_to_heraldry(colour,width.is_some()) {
                        ShapeCategory::Heraldry(heraldry.canvases_used(),heraldry.scale(),prio)                                
                    } else {
                        ShapeCategory::Solid(prio)
                    }
                });
                for (pkind,filter) in &mut demerge_colour {
                    filter.set_size(area.len());
                    match pkind {
                        ShapeCategory::Solid(prio) => {
                            out.push(GLShape::SpaceBaseRect(area.filter(filter),SimpleShapePatina::from_patina(patina.filter(filter))?,filter.filter(&allotment),kind.clone(),*prio));
                        },
                        ShapeCategory::Heraldry(HeraldryCanvasesUsed::Solid(heraldry_canvas),scale,prio) => {
                            let heraldry_tool = tools.heraldry();
                            let mut heraldry = make_heraldry(patina.filter(filter))?;
                            let handles = heraldry.drain(..).map(|x| heraldry_tool.add(x)).collect::<Vec<_>>();
                            let area = area.filter(filter);
                            let allotment = filter.filter(&allotment);
                            out.push(GLShape::Heraldry(area,handles,allotment,kind.clone(),heraldry_canvas.clone(),scale.clone(),None,*prio));
                        },
                        ShapeCategory::Heraldry(HeraldryCanvasesUsed::Hollow(heraldry_canvas_h,heraldry_canvas_v),scale,prio) => {
                            let width = width.unwrap_or(0) as f64;
                            let heraldry_tool = tools.heraldry();
                            let mut heraldry = make_heraldry(patina.filter(filter))?;
                            let handles = heraldry.drain(..).map(|x| heraldry_tool.add(x)).collect::<Vec<_>>();
                            let area = area.filter(filter);
                            let allotment = filter.filter(&allotment);
                            // XXX too much cloning, at least Arc them
                            out.push(GLShape::Heraldry(area.clone(),handles.clone(),allotment.clone(),kind.clone(),heraldry_canvas_v.clone(),scale.clone(),Some(HollowEdge::Left(width)),*prio));
                            out.push(GLShape::Heraldry(area.clone(),handles.clone(),allotment.clone(),kind.clone(),heraldry_canvas_v.clone(),scale.clone(),Some(HollowEdge::Right(width)),*prio));
                            out.push(GLShape::Heraldry(area.clone(),handles.clone(),allotment.clone(),kind.clone(),heraldry_canvas_h.clone(),scale.clone(),Some(HollowEdge::Top(width)),*prio));
                            out.push(GLShape::Heraldry(area.clone(),handles,allotment,kind.clone(),heraldry_canvas_h.clone(),scale.clone(),Some(HollowEdge::Bottom(width)),*prio));
                        }
                    }
                }        
            },
            PatinaExtract::ZMenu(zmenu,values) => {
                out.push(GLShape::SpaceBaseRect(area,SimpleShapePatina::ZMenu(zmenu,values),allotment,kind.clone(),0));
            }
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
        Patina::Filled(c,_) => (c,false),
        Patina::Hollow(c,_,_) => (c,true),
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

pub(crate) fn prepare_shape_in_layer(_layer: &mut Layer, tools: &mut DrawingTools, shape: Shape, allotter: &Allotter) -> Result<Vec<GLShape>,Message> {
    Ok(match shape {
        Shape::Wiggle(range,y,plotter,allotment) => {
            let allotment = allotments(allotter,&[allotment])?;
            vec![GLShape::Wiggle(range,y,plotter,allotment[0].clone(),0)]
        },
        Shape::Text(spacebase,pen,texts,allotment,kind) => {
            let allotment = allotments(allotter,&allotment)?;
            let drawing_text = tools.text();
            let colours_iter = pen.colours().iter().cycle();
            let background = pen.background();
            let handles : Vec<_> = texts.iter().zip(colours_iter).map(|(text,colour)| drawing_text.add_text(&pen,text,colour,background)).collect();
            vec![GLShape::Text(spacebase,handles,allotment,AllotmentProgram::new(&kind).kind(),pen.depth())]
        },
        Shape::Image(spacebase,depth,images,allotment,kind) => {
            let allotment = allotments(allotter,&allotment)?;
            let drawing_bitmap = tools.bitmap();
            let handles = images.iter().map(|asset| drawing_bitmap.add_bitmap(asset)).collect::<Vec<_>>();
            vec![GLShape::Image(spacebase,handles,allotment,AllotmentProgram::new(&kind).kind(),depth)]
        },
        Shape::SpaceBaseRect(area,patina,allotment,kind) => {
            split_spacebaserect(tools,allotter,area,patina,allotment)?
        }
    })
}
