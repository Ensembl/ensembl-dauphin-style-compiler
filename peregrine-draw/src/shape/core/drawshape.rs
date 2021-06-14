use peregrine_data::{
    Allotment, AllotmentHandle, Allotter, Colour, DirectColour, Patina, Plotter,
    Shape, SpaceBaseArea, SpaceBase, AllotmentPositionKind, PositionVariant, DataFilter
};
use super::text::TextHandle;
use super::super::layers::layer::{ Layer };
use super::super::layers::patina::{ PatinaProgramName, PatinaProcessName };
use super::super::layers::geometry::GeometryProgramName;
use super::texture::CanvasTextureAreas;
use crate::webgl::{DrawingAllFlatsBuilder, ProcessStanzaAddable, ProcessStanzaArray, ProcessStanzaElements };
use crate::webgl::global::WebGlGlobal;
use super::super::layers::drawing::DrawingTools;
use crate::util::message::Message;
use super::tracktriangles::TrianglesKind;
use super::heraldry::HeraldryHandle;

#[derive(Clone,PartialEq,Eq,Hash)]
pub enum AllotmentProgramKind {
    Track,
    BaseLabel,
    SpaceLabel
}

#[derive(Clone,PartialEq,Eq,Hash,Debug)]
pub enum PatinaProgram {
    Solid,
    Striped
}

pub(crate) enum SimpleShapePatina {
    Solid(Vec<DirectColour>),
    Hollow(Vec<DirectColour>)
}

fn simplify_colours(mut colours: Vec<Colour>) -> Result<Vec<DirectColour>,Message> {
    colours.drain(..).map(|colour| {
        match colour {
            Colour::Direct(d) => Ok(d),
            Colour::Stripe(_,_,_) => Err(Message::CodeInvariantFailed(format!("attempt to simplify stripe to colour")))
        }
    }).collect::<Result<Vec<_>,_>>()
}

impl SimpleShapePatina {
    pub(crate) fn from_patina(patina: Patina) -> Result<SimpleShapePatina,Message> {
        Ok(match patina {
            Patina::Filled(colours) => { SimpleShapePatina::Solid(simplify_colours(colours)?) },
            Patina::Hollow(colours) => { SimpleShapePatina::Hollow(simplify_colours(colours)?) },
            _ => Err(Message::CodeInvariantFailed(format!("attempt to simplify nonfill.hollow to colour")))?
        })
    }
}

pub(crate) enum GLShape {
    Text2(SpaceBase,Vec<TextHandle>,Vec<Allotment>,AllotmentProgramKind),
    Heraldry(SpaceBaseArea,Vec<HeraldryHandle>,Vec<Allotment>,AllotmentProgramKind),
    Wiggle((f64,f64),Vec<Option<f64>>,Plotter,Allotment),
    SpaceBaseRect(SpaceBaseArea,SimpleShapePatina,Vec<Allotment>,AllotmentProgramKind),
}
pub enum AllotmentProgram {
    Track,
    BaseLabel(PositionVariant),
    SpaceLabel(PositionVariant)
}

impl AllotmentProgram {
    pub(super) fn kind(&self) -> AllotmentProgramKind {
        match self {
            AllotmentProgram::Track => AllotmentProgramKind::Track,
            AllotmentProgram::SpaceLabel(_) => AllotmentProgramKind::SpaceLabel,
            AllotmentProgram::BaseLabel(_) => AllotmentProgramKind::BaseLabel
        }        
    }
}

impl AllotmentProgram {
    pub(super) fn new(allotment: &AllotmentPositionKind) -> AllotmentProgram {
        match allotment {
            AllotmentPositionKind::Track => AllotmentProgram::Track,
            AllotmentPositionKind::SpaceLabel(x) => AllotmentProgram::SpaceLabel(x.clone()),
            AllotmentPositionKind::BaseLabel(x) => AllotmentProgram::BaseLabel(x.clone())
        }
    }
}

fn add_colour(addable: &mut dyn ProcessStanzaAddable, layer: &mut Layer, geometry: &GeometryProgramName, patina: &SimpleShapePatina) -> Result<(),Message> {
    let vertexes = match patina {
        SimpleShapePatina::Solid(_) => 4,
        SimpleShapePatina::Hollow(_) => 8,
        _ => 0
    };
    match patina {
        SimpleShapePatina::Solid(colours) | SimpleShapePatina::Hollow(colours) => {
            let direct = layer.get_direct(geometry)?;
            direct.direct(addable,&colours,vertexes)?;
        },
        /*
        Patina::Filled(Colour::Spot(d)) |  Patina::Hollow(Colour::Spot(d)) => {
            let spot = layer.get_spot(geometry,d)?;
            let mut process = layer.get_process_mut(geometry,&PatinaProcessName::Spot(d.clone()))?;
            spot.spot(&mut process)?;
        }
        Patina::ZMenu(template,values) => {
            // XXX ZMenu
            // tools.zmenus().add_rectangle(layer,zmenu,values,anchor,allotment,x_size,y_size);
        }
        */
    }
    Ok(())
}

fn to_trianges_kind(program_kind: &AllotmentProgramKind) -> TrianglesKind {
    match program_kind {
        AllotmentProgramKind::Track => TrianglesKind::Track,
        AllotmentProgramKind::BaseLabel => TrianglesKind::Base,
        AllotmentProgramKind::SpaceLabel => TrianglesKind::Space
    }
}

fn position_canvas_areas(position: &SpaceBase, areas: &[CanvasTextureAreas]) -> SpaceBaseArea {
    let mut x_sizes = vec![];
    let mut y_sizes = vec![];
    for dim in areas {
        let size = dim.size();
        x_sizes.push(size.0 as f64);
        y_sizes.push(size.1 as f64);
    }            
    SpaceBaseArea::new_from_sizes(&position,&x_sizes,&y_sizes)
}

pub(crate) fn add_shape_to_layer(layer: &mut Layer, gl: &WebGlGlobal,  tools: &mut DrawingTools, canvas_builder: &DrawingAllFlatsBuilder, shape: GLShape) -> Result<(),Message> {
    match shape {
        GLShape::Wiggle((start,end),y,Plotter(height,colour),allotment) => {
            //let patina = colour_to_patina(&Colour::Spot(colour.clone()));
            //let (mut array,geometry) = add_wiggle(layer,start,end,y,height,&patina,allotment)?;
            //let spot = layer.get_spot(&geometry,&colour)?;
            //let mut process = layer.get_process_mut(&geometry,&patina)?;
            //spot.spot(&mut process)?;
            //array.close();
        },
        GLShape::Text2(position,handles,allotments,program_kind) => {
            let kind = to_trianges_kind(&program_kind);
            // TODO factor
            let text = tools.text();
            let dims = handles.iter()
                .map(|handle| text.get_texture_areas(handle))
                .collect::<Result<Vec<_>,_>>()?;
            let area = position_canvas_areas(&position,&dims);
            let left = layer.left();
            let canvas = text.canvas_id()?;
            let geometry = kind.geometry_program_name();
            let patina = layer.get_texture(&geometry,&canvas)?;
            let track_triangles = kind.get_process(layer,&PatinaProcessName::Texture(canvas.clone()))?;
            let builder = layer.get_process_mut(&geometry, &PatinaProcessName::Texture(canvas.clone()))?;
            let campaign = track_triangles.add_rectangles(builder,&area,&allotments,left,false,kind)?;
            if let Some(mut campaign) = campaign {
                patina.add_rectangle(builder,&mut campaign,gl.bindery(),&canvas,&dims,gl.flat_store())?;
                campaign.close();
            }
        },
        GLShape::Heraldry(area,handles,allotments,program_kind) => {
            let kind = to_trianges_kind(&program_kind);
            let left = layer.left();
            let heraldry = tools.heraldry();
            let dims = handles.iter()
                .map(|handle| heraldry.get_texture_areas(handle))
                .collect::<Result<Vec<_>,_>>()?;
            let canvas = heraldry.canvas_id()?;
            let geometry = kind.geometry_program_name();
            let patina = layer.get_texture(&geometry,&canvas)?;
            let track_triangles = kind.get_process(layer,&PatinaProcessName::Texture(canvas.clone()))?;
            let builder = layer.get_process_mut(&kind.geometry_program_name(),&PatinaProcessName::Texture(canvas.clone()))?;
            let campaign = track_triangles.add_rectangles(builder, &area, &allotments,left,false,kind)?;
            if let Some(mut campaign) = campaign {
                patina.add_rectangle(builder,&mut campaign,gl.bindery(),&canvas,&dims,gl.flat_store())?;
                campaign.close();
            }
        },
        GLShape::SpaceBaseRect(area,patina,allotments,allotment_kind) => {
            let kind = to_trianges_kind(&allotment_kind);
            let left = layer.left();
            let track_triangles = kind.get_process(layer,&PatinaProcessName::Direct)?;
            let builder = layer.get_process_mut(&kind.geometry_program_name(),&PatinaProcessName::Direct)?;
            let hollow = match patina { SimpleShapePatina::Hollow(_) => true, _ => false };
            let campaign = track_triangles.add_rectangles(builder, &area, &allotments,left,hollow,kind)?;
            if let Some(mut campaign) = campaign {
                add_colour(&mut campaign,layer,&GeometryProgramName::TrackTriangles,&patina)?;
                campaign.close();
            }
            // XXX ZMenus
        }
    }
    Ok(())
}
