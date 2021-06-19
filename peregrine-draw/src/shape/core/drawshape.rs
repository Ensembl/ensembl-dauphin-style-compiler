use peregrine_data::{
    Allotment, AllotmentPositionKind, Colour, DataFilterBuilder, DirectColour, Patina, Plotter, PositionVariant, SpaceBase, 
    SpaceBaseArea, expand_by_repeating
};
use super::text::TextHandle;
use super::super::layers::layer::{ Layer };
use super::super::layers::patina::{ PatinaProcessName };
use super::super::layers::geometry::GeometryProgramName;
use super::texture::{CanvasTextureArea};
use crate::shape::core::heraldry::HeraldryCanvas;
use crate::webgl::{ ProcessStanzaAddable };
use crate::webgl::global::WebGlGlobal;
use super::super::layers::drawing::DrawingTools;
use crate::util::message::Message;
use super::tracktriangles::TrianglesKind;
use super::heraldry::{HeraldryHandle, HeraldryScale};
use crate::webgl::canvas::flatstore::FlatId;

#[derive(Clone,PartialEq,Eq,Hash)]
pub enum AllotmentProgramKind {
    Track,
    BaseLabel,
    SpaceLabel
}

pub(crate) enum SimpleShapePatina {
    Solid(Vec<DirectColour>),
    Hollow(Vec<DirectColour>)
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
            Patina::Filled(colours) => { SimpleShapePatina::Solid(simplify_colours(colours)?) },
            Patina::Hollow(colours) => { SimpleShapePatina::Hollow(simplify_colours(colours)?) },
            _ => Err(Message::CodeInvariantFailed(format!("attempt to simplify nonfill.hollow to colour")))?
        })
    }
}

pub(crate) enum GLShape {
    Text2(SpaceBase,Vec<TextHandle>,Vec<Allotment>,AllotmentProgramKind),
    Heraldry(SpaceBaseArea,Vec<HeraldryHandle>,Vec<Allotment>,AllotmentProgramKind,HeraldryCanvas,HeraldryScale),
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

fn position_canvas_areas(position: &SpaceBase, areas: &[CanvasTextureArea]) -> SpaceBaseArea {
    let mut x_sizes = vec![];
    let mut y_sizes = vec![];
    for dim in areas {
        let size = dim.size();
        x_sizes.push(size.0 as f64);
        y_sizes.push(size.1 as f64);
    }            
    SpaceBaseArea::new_from_sizes(&position,&x_sizes,&y_sizes)
}

fn draw_from_canvas(layer: &mut Layer, gl: &WebGlGlobal, kind: &TrianglesKind, area: &SpaceBaseArea, allotments: &[Allotment], canvas: &FlatId, mut dims: &[CanvasTextureArea]) -> Result<(),Message> {
    let geometry = kind.geometry_program_name();
    let left = layer.left();
    let patina = layer.get_free_texture(&geometry,&canvas)?;
    let track_triangles = kind.get_process(layer,&PatinaProcessName::FreeTexture(canvas.clone()))?;
    let builder = layer.get_process_mut(&kind.geometry_program_name(),&PatinaProcessName::FreeTexture(canvas.clone()))?;
    /**/
    let mut campaign = track_triangles.add_rectangles(builder,area,allotments,left,false,&kind)?;
    patina.add_rectangle(&mut campaign,&canvas,&dims,gl.flat_store())?;
    campaign.close();
    Ok(())
}

fn overrun_horiz(area_a: &SpaceBaseArea, dims: &mut Vec<CanvasTextureArea>) {
    //expand_by_repeating(dims,area_a.len());
}

fn draw_heraldry_canvas(layer: &mut Layer, gl: &WebGlGlobal, tools: &mut DrawingTools, kind: &TrianglesKind, area_a: &SpaceBaseArea, handles: &[HeraldryHandle], allotments: &[Allotment], heraldry_canvas: &HeraldryCanvas, scale: &HeraldryScale) -> Result<(),Message> {
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
    if filter.count() == 0 { return Ok(()); }
    let canvas = heraldry.canvas_id(&heraldry_canvas).ok_or_else(|| Message::CodeInvariantFailed("no canvas id A".to_string()))?;
    if scale.overrun_horiz(heraldry_canvas) {
        overrun_horiz(area_a, &mut dims);
    }
    draw_from_canvas(layer,gl,kind,&area_a.filter(&filter),allotments,&canvas,&dims)?;
    Ok(())
}

pub(crate) fn add_shape_to_layer(layer: &mut Layer, gl: &WebGlGlobal,  tools: &mut DrawingTools, shape: GLShape) -> Result<(),Message> {
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
                .map(|handle| text.manager().get_texture_areas(handle))
                .collect::<Result<Vec<_>,_>>()?;
            let area = position_canvas_areas(&position,&dims);
            let canvas = text.manager().canvas_id().ok_or_else(|| Message::CodeInvariantFailed("no canvas id A".to_string()))?;
            draw_from_canvas(layer,gl,&kind,&area,&allotments,&canvas,&dims)?;
        },
        GLShape::Heraldry(area,handles,allotments,program_kind,heraldry_canvas,scale) => {
            let kind = to_trianges_kind(&program_kind);
            draw_heraldry_canvas(layer,gl,tools,&kind,&area,&handles,&allotments,&heraldry_canvas,&scale)?;
        },
        GLShape::SpaceBaseRect(area,patina,allotments,allotment_kind) => {
            let kind = to_trianges_kind(&allotment_kind);
            let left = layer.left();
            let track_triangles = kind.get_process(layer,&PatinaProcessName::Direct)?;
            let builder = layer.get_process_mut(&kind.geometry_program_name(),&PatinaProcessName::Direct)?;
            let hollow = match patina { SimpleShapePatina::Hollow(_) => true, _ => false };
            let mut campaign = track_triangles.add_rectangles(builder, &area, &allotments,left,hollow,&kind)?;
            add_colour(&mut campaign,layer,&GeometryProgramName::TrackTriangles,&patina)?;
            campaign.close();
            // XXX ZMenus
        }
    }
    Ok(())
}
