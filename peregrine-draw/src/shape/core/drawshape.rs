use peregrine_data::{
    Allotment, AllotmentHandle, Allotter, Colour, DirectColour, Patina, Plotter,
    Shape, SpaceBaseArea, SpaceBase, AllotmentPositionKind, PositionVariant, DataFilter
};
use super::text::TextHandle;
use super::super::layers::layer::{ Layer };
use super::super::layers::patina::{ PatinaProgramName, PatinaProcessName };
use super::super::layers::geometry::GeometryProgramName;
use super::texture::CanvasTextureAreas;
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

#[derive(Clone,PartialEq,Eq,Hash,Debug)]
pub enum PatinaProgram {
    Solid,
    Striped
}

pub enum GLShape {
    Text2(SpaceBase,Vec<TextHandle>,Vec<Allotment>,AllotmentProgramKind),
    Wiggle((f64,f64),Vec<Option<f64>>,Plotter,Allotment),
    SpaceBaseRect(SpaceBaseArea,Patina,Vec<Allotment>,AllotmentProgramKind,PatinaProgram),
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


fn xxx_destripe(c: &[Colour]) -> Vec<DirectColour> {
    c.iter().map(|colour| match colour {
        Colour::Direct(d) => d,
        Colour::Stripe(a,b) => a
    }).cloned().collect()
}

fn add_colour(addable: &mut dyn ProcessStanzaAddable, layer: &mut Layer, geometry: &GeometryProgramName, patina: &Patina) -> Result<(),Message> {
    let vertexes = match patina {
        Patina::Filled(_) => 4,
        Patina::Hollow(_) => 8,
        _ => 0
    };
    match patina {
        Patina::Filled(colours) | Patina::Hollow(colours) => {
            let d = xxx_destripe(colours);
            let direct = layer.get_direct(geometry)?;
            direct.direct(addable,&d,vertexes)?;
        },
        /*
        Patina::Filled(Colour::Spot(d)) |  Patina::Hollow(Colour::Spot(d)) => {
            let spot = layer.get_spot(geometry,d)?;
            let mut process = layer.get_process_mut(geometry,&PatinaProcessName::Spot(d.clone()))?;
            spot.spot(&mut process)?;
        }
        */
        Patina::ZMenu(template,values) => {
            // XXX ZMenu
            // tools.zmenus().add_rectangle(layer,zmenu,values,anchor,allotment,x_size,y_size);
        }
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

pub(crate) fn add_shape_to_layer(layer: &mut Layer, gl: &WebGlGlobal,  tools: &mut DrawingTools, canvas_builder: &DrawingFlatsDrawable, shape: GLShape) -> Result<(),Message> {
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
            let canvas = text.canvas_id(canvas_builder)?;
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
        GLShape::SpaceBaseRect(area,patina,allotments,allotment_kind,PatinaProgram::Solid) => {
            let kind = to_trianges_kind(&allotment_kind);
            let left = layer.left();
            let patina_name = PatinaProcessName::from_patina(&patina);
            if let Some(patina_name) = patina_name {
                let track_triangles = kind.get_process(layer,&patina_name)?;
                let builder = layer.get_process_mut(&kind.geometry_program_name(),&patina_name)?;
                let hollow = match patina { Patina::Hollow(_) => true, _ => false };
                let campaign = track_triangles.add_rectangles(builder, &area, &allotments,left,hollow,kind)?;
                if let Some(mut campaign) = campaign {
                    add_colour(&mut campaign,layer,&GeometryProgramName::TrackTriangles,&patina)?;
                    campaign.close();
                }
            }
            // XXX ZMenus
        }
        GLShape::SpaceBaseRect(area,patina,allotments,allotment_kind,PatinaProgram::Striped) => {
            let kind = to_trianges_kind(&allotment_kind);
            let left = layer.left();
            let patina_name = PatinaProcessName::from_patina(&patina);
            if let Some(patina_name) = patina_name {
                let track_triangles = kind.get_process(layer,&patina_name)?;
                let builder = layer.get_process_mut(&kind.geometry_program_name(),&patina_name)?;
                let hollow = match patina { Patina::Hollow(_) => true, _ => false };
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
