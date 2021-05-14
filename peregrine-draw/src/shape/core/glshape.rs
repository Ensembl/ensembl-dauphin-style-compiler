use peregrine_data::{
    Allotment, AllotmentHandle, Allotter, Colour, DirectColour, Patina, Plotter,
    Shape, SpaceBaseArea, SpaceBase, AllotmentPositionKind, PositionVariant, DataFilter
};
use super::text::TextHandle;
use super::super::layers::layer::{ Layer };
use super::super::layers::patina::{ PatinaProgramName, PatinaProcessName };
use super::super::layers::geometry::GeometryProgramName;
use crate::webgl::{DrawingFlatsDrawable, ProcessStanzaAddable, ProcessStanzaArray, ProcessStanzaElements };
use crate::webgl::global::WebGlGlobal;
use super::super::layers::drawing::DrawingTools;
use crate::util::message::Message;
use super::tracktriangles::TrianglesKind;

#[derive(Clone,PartialEq,Eq,Hash)]
pub enum AllotmentProgramKind {
    Track,
    BaseLabel,
    SpaceLabel
}

pub enum AllotmentProgram {
    Track,
    BaseLabel(PositionVariant),
    SpaceLabel(PositionVariant)
}

impl AllotmentProgram {
    fn kind(&self) -> AllotmentProgramKind {
        match self {
            AllotmentProgram::Track => AllotmentProgramKind::Track,
            AllotmentProgram::SpaceLabel(_) => AllotmentProgramKind::SpaceLabel,
            AllotmentProgram::BaseLabel(_) => AllotmentProgramKind::BaseLabel
        }        
    }
}

impl AllotmentProgram {
    fn new(allotment: &AllotmentPositionKind) -> AllotmentProgram {
        match allotment {
            AllotmentPositionKind::Track => AllotmentProgram::Track,
            AllotmentPositionKind::SpaceLabel(x) => AllotmentProgram::SpaceLabel(x.clone()),
            AllotmentPositionKind::BaseLabel(x) => AllotmentProgram::BaseLabel(x.clone())
        }
    }
}

pub enum PreparedShape {
    Text2(SpaceBase,Vec<TextHandle>,Vec<Allotment>,AllotmentProgramKind),
    Wiggle((f64,f64),Vec<Option<f64>>,Plotter,Allotment),
    SpaceBaseRect(SpaceBaseArea,Patina,Vec<Allotment>,AllotmentProgramKind),
}

fn colour_to_patina(colour: &Colour) -> PatinaProcessName {
    match colour {
        Colour::Direct(_) => PatinaProcessName::Direct,
        Colour::Spot(c) => PatinaProcessName::Spot(c.clone())
    }
}

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

fn add_colour(addable: &mut dyn ProcessStanzaAddable, layer: &mut Layer, geometry: &GeometryProgramName, patina: &Patina) -> Result<(),Message> {
    let vertexes = match patina {
        Patina::Filled(_) => 4,
        Patina::Hollow(_) => 8,
        _ => 0
    };
    match patina {
        Patina::Filled(Colour::Direct(d)) | Patina::Hollow(Colour::Direct(d)) => {
            let direct = layer.get_direct(geometry)?;
            direct.direct(addable,d,vertexes)?;
        },
        Patina::Filled(Colour::Spot(d)) |  Patina::Hollow(Colour::Spot(d)) => {
            let spot = layer.get_spot(geometry,d)?;
            let mut process = layer.get_process_mut(geometry,&PatinaProcessName::Spot(d.clone()))?;
            spot.spot(&mut process)?;
        }
        Patina::ZMenu(template,values) => {
            // XXX ZMenu
            // tools.zmenus().add_rectangle(layer,zmenu,values,anchor,allotment,x_size,y_size);
        }
    }
    Ok(())
}

// XXX not a new one for each!
fn allotments(allotter: &Allotter, allotments: &[AllotmentHandle]) -> Result<Vec<Allotment>,Message> {
    allotments.iter().map(|handle| {
        allotter.get(handle).map(|a| a.clone())
    }).collect::<Result<Vec<_>,_>>().map_err(|e| Message::DataError(e))
}

pub(crate) fn prepare_shape_in_layer(_layer: &mut Layer, tools: &mut DrawingTools, shape: Shape, allotter: &Allotter) -> Result<Vec<PreparedShape>,Message> {
    Ok(match shape {
        Shape::Wiggle(range,y,plotter,allotment) => {
            let allotment = allotments(allotter,&[allotment])?;
            vec![PreparedShape::Wiggle(range,y,plotter,allotment[0].clone())]
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
                out.push(PreparedShape::Text2(spacebase.filter(filter),filter.filter(&handles),filter.filter(&allotment),kind.clone()));
            }
            out
        },
        Shape::SpaceBaseRect(area,patina,allotment) => {
            let allotment = allotments(allotter,&allotment)?;
            let demerge = DataFilter::demerge(&allotment,|allotment| {
                AllotmentProgram::new(&allotment.position().kind()).kind()
            });
            let mut out = vec![];
            for (kind,filter) in &demerge {
                out.push(PreparedShape::SpaceBaseRect(area.filter(filter),patina.filter2(filter),filter.filter(&allotment),kind.clone()));
            }
            out
        }
    })
}

pub(crate) fn add_shape_to_layer(layer: &mut Layer, gl: &WebGlGlobal,  tools: &mut DrawingTools, canvas_builder: &DrawingFlatsDrawable, shape: PreparedShape) -> Result<(),Message> {
    match shape {
        PreparedShape::Wiggle((start,end),y,Plotter(height,colour),allotment) => {
            //let patina = colour_to_patina(&Colour::Spot(colour.clone()));
            //let (mut array,geometry) = add_wiggle(layer,start,end,y,height,&patina,allotment)?;
            //let spot = layer.get_spot(&geometry,&colour)?;
            //let mut process = layer.get_process_mut(&geometry,&patina)?;
            //spot.spot(&mut process)?;
            //array.close();
        },
        PreparedShape::Text2(position,handles,allotments,program_kind) => {
            // TODO factor
            let kind = match program_kind {
                AllotmentProgramKind::Track => TrianglesKind::Track,
                AllotmentProgramKind::BaseLabel => TrianglesKind::Base,
                AllotmentProgramKind::SpaceLabel => TrianglesKind::Space
            };
            let text = tools.text();
            let mut dims = vec![];
            let mut x_sizes = vec![];
            let mut y_sizes = vec![];
            let canvas = text.canvas_id(canvas_builder)?;
            for handle in &handles {
                let texture_areas = text.get_texture_areas(handle)?;
                let size = texture_areas.size();
                x_sizes.push(size.0 as f64);
                y_sizes.push(size.1 as f64);
                dims.push(texture_areas);
            }
            let left = layer.left();
            let geometry = kind.geometry_program_name();
            let patina = layer.get_texture(&geometry,&canvas)?;
            let area = SpaceBaseArea::new_from_sizes(&position,&x_sizes,&y_sizes);
            let track_triangles = kind.get_process(layer,&PatinaProcessName::Texture(canvas.clone()))?;
            let builder = layer.get_process_mut(&kind.geometry_program_name(), &PatinaProcessName::Texture(canvas.clone()))?;
            let campaign = track_triangles.add_rectangles(builder,&area,&allotments,left,false,kind)?;
            if let Some(mut campaign) = campaign {
                patina.add_rectangle(builder,&mut campaign,gl.bindery(),&canvas,&dims,gl.flat_store())?;
                campaign.close();
            }
        },
        PreparedShape::SpaceBaseRect(area,patina,allotments,program_kind) => {
            let kind = match program_kind {
                AllotmentProgramKind::Track => TrianglesKind::Track,
                AllotmentProgramKind::BaseLabel => TrianglesKind::Base,
                AllotmentProgramKind::SpaceLabel => TrianglesKind::Space
            };
            let left = layer.left();
            let patina_name = PatinaProcessName::from_patina(&patina);
            if let Some(patina_name) = patina_name {
                let track_triangles = kind.get_process(layer,&patina_name)?;
                let builder = layer.get_process_mut(&kind.geometry_program_name(),&patina_name)?;
                let hollow = match patina {
                    Patina::Hollow(_) => true,
                    _ => false
                };
                let campaign = track_triangles.add_rectangles(builder, &area, &allotments,left,hollow,kind)?;
                if let Some(mut campaign) = campaign {
                    add_colour(&mut campaign,layer,&GeometryProgramName::TrackTriangles,&patina)?;
                    campaign.close();
                }
            }
            // XXX ZMenus
        }
    }
    Ok(())
}