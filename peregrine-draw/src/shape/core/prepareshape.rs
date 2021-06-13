use peregrine_data::{
    Allotment, AllotmentHandle, Allotter, Colour, Patina, Plotter,
    Shape, DataFilter
};
use super::text::TextHandle;
use super::super::layers::layer::{ Layer };
use super::super::layers::drawing::DrawingTools;
use crate::util::message::Message;
use super::tracktriangles::TrianglesKind;
use super::drawshape::{ GLShape, AllotmentProgramKind, AllotmentProgram, PatinaProgram };

/*
fn colour_to_patina(colour: &Colour) -> PatinaProcessName {
    match colour {
        Colour::Direct(_) => PatinaProcessName::Direct,
        Colour::Spot(c) => PatinaProcessName::Spot(c.clone()),
        //Colour::Stripe(_,_) => PatinaProcessName::Texture(FlatId::)
    }
}
*/

fn apply_allotments(y: &[f64], allotment: &[Allotment]) -> Vec<f64> {
    // XXX yuk!
    let len = if y.len() != allotment.len() {
        y.len() * allotment.len()
    } else {
        y.len()
    };
    let mut iter = allotment.iter().cycle().zip(y.iter().cycle());
    (0..len).map(|_| {
        let (allotment,y) = iter.next().unwrap();
        let offset = allotment.position().offset() as f64;
        *y+offset
    }).collect()
}

    /* TODO fix
fn add_wiggle<'a>(layer: &'a mut Layer, start: f64, end: f64, y: Vec<Option<f64>>, height: f64, patina: &PatinaProcessName, _allotment: Allotment) -> Result<(ProcessStanzaArray,GeometryProgramName),Message> {    
    let wiggle = layer.get_wiggle(patina)?;
    let left = layer.left();
    let process = layer.get_process_mut(&GeometryProgramName::Page,patina)?;
    let stanza_builder = wiggle.add_wiggle(process,start,end,y,height,left)?;
    Ok((stanza_builder,GeometryProgramName::Pin))
}
*/

// XXX not a new one for each!
fn allotments(allotter: &Allotter, allotments: &[AllotmentHandle]) -> Result<Vec<Allotment>,Message> {
    allotments.iter().map(|handle| {
        allotter.get(handle).map(|a| a.clone())
    }).collect::<Result<Vec<_>,_>>().map_err(|e| Message::DataError(e))
}

pub(crate) fn prepare_shape_in_layer(_layer: &mut Layer, tools: &mut DrawingTools, shape: Shape, allotter: &Allotter) -> Result<Vec<GLShape>,Message> {
    Ok(match shape {
        Shape::Wiggle(range,y,plotter,allotment) => {
            let allotment = allotments(allotter,&[allotment])?;
            vec![GLShape::Wiggle(range,y,plotter,allotment[0].clone())]
        },
        Shape::Text2(spacebase,mut pen,texts,allotment) => {
            let allotment = allotments(allotter,&allotment)?;
            let demerge = DataFilter::demerge(&allotment,|allotment| {
                AllotmentProgram::new(&allotment.position().kind()).kind()
            });
            let drawing_text = tools.text();
            let colours_iter = pen.2.iter().cycle();
            let handles : Vec<_> = texts.iter().zip(colours_iter).map(|(text,colour)| drawing_text.add_text(&pen,text,colour)).collect();
            let mut out = vec![];
            for (kind,filter) in &demerge {
                out.push(GLShape::Text2(spacebase.filter(filter),filter.filter(&handles),filter.filter(&allotment),kind.clone()));
            }
            out
        },
        Shape::SpaceBaseRect(area,patina,allotment) => {
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
                let xxx = vec![];
                let colours = match &patina {
                    Patina::Filled(c) => c,
                    Patina::Hollow(c) => c,
                    Patina::ZMenu(_,_) => &xxx // XXX zmenus 
                };
                let mut demerge_colour = DataFilter::demerge(&colours,|colour| {
                    match colour {
                        Colour::Direct(_) => PatinaProgram::Solid,
                        Colour::Stripe(_,_) => PatinaProgram::Striped
                    }
                });
                for (pkind,filter) in &mut demerge_colour {
                    filter.set_size(area.len());
                    out.push(GLShape::SpaceBaseRect(area.filter(filter),patina.filter(filter),filter.filter(&allotment),kind.clone(),pkind.clone()));
                }
            }
            out
        }
    })
}
