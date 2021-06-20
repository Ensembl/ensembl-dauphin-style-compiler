use std::{borrow::Borrow, collections::HashMap};
use super::super::core::wigglegeometry::WiggleProgram;
use super::super::core::directcolourdraw::DirectColourDraw;
use super::super::core::spotcolourdraw::SpotColourDraw;
use super::super::core::tracktriangles::TrackTrianglesProgram;
use super::super::core::texture::TextureDraw;
use crate::{ webgl::FlatId};
use crate::webgl::{ ProcessBuilder, Process, DrawingAllFlats };
use super::geometry::{ GeometryProgramName, GeometryProgram };
use super::programstore::ProgramStore;
use super::patina::{ PatinaProcess, PatinaProcessName };
use peregrine_data::DirectColour;
use crate::util::message::Message;
use crate::webgl::global::WebGlGlobal;
use super::shapeprogram::ShapeProgram;

use crate::force_branch;

/* 
TODO ensure + index
TODO y split bug
TODO y from bottom
TODO layers from core
TODO ordered layers
TODO remove datum option from stretchtangles
TODO return shapes from core without cloning (drain)
TODO uniforms set only on change
TODO global destroy
TODO keep program when same program
TODO initial clear
TODO wiggle width
TODO hollowwidth
TODO intersection cache
*/

#[derive(Clone,PartialEq,Eq,Hash)]
pub(crate) struct ProgramCharacter(pub GeometryProgramName, pub PatinaProcessName);

pub(crate) struct Layer {
    programs: ProgramStore,
    store: HashMap<ProgramCharacter,ShapeProgram>,
    left: f64
}

macro_rules! layer_geometry_accessor {
    ($func:ident,$geom_type:ty,$geom_name:ident) => {
        pub(crate) fn $func(&mut self, patina: &PatinaProcessName) -> Result<$geom_type,Message> {
            let geom = self.shape_program(&GeometryProgramName::$geom_name,patina)?.get_geometry();
            force_branch!(GeometryProgram,$geom_name,geom)
        }
    };
}

macro_rules! layer_patina_accessor {
    ($func:ident,$patina_type:ty,$patina_name:ident) => {
        pub(crate) fn $func(&mut self, geometry: &GeometryProgramName) -> Result<$patina_type,Message> {
            let patina = self.shape_program(geometry,&PatinaProcessName::$patina_name)?.get_patina();
            force_branch!(PatinaProcess,$patina_name,patina)
        }                
    };
}

impl Layer {
    pub fn new(programs: &ProgramStore, left: f64) -> Result<Layer,Message> {
        Ok(Layer {
            programs: programs.clone(),
            store: HashMap::new(),
            left
        })
    }

    pub(crate) fn left(&self) -> f64 { self.left }

    pub(crate) fn shape_program(&mut self, geometry: &GeometryProgramName, patina: &PatinaProcessName) -> Result<&mut ShapeProgram,Message> {
        let character = ProgramCharacter(geometry.clone(),patina.clone());
        if !self.store.contains_key(&character) {
            self.store.insert(character.clone(),ShapeProgram::new(&self.programs,&geometry,&patina)?);
        }
        Ok(self.store.get_mut(&character).unwrap())
    }

    layer_geometry_accessor!(get_wiggle,WiggleProgram,Wiggle);
    layer_geometry_accessor!(get_track_triangles,TrackTrianglesProgram,TrackTriangles);
    layer_geometry_accessor!(get_base_label_triangles,TrackTrianglesProgram,BaseLabelTriangles);
    layer_geometry_accessor!(get_space_label_triangles,TrackTrianglesProgram,SpaceLabelTriangles);

    layer_patina_accessor!(get_direct,DirectColourDraw,Direct);

    pub(crate) fn get_spot(&mut self, geometry: &GeometryProgramName, colour: &DirectColour) -> Result<SpotColourDraw,Message> {
        let patina = self.shape_program(geometry,&PatinaProcessName::Spot(colour.clone()))?.get_patina();
        force_branch!(PatinaProcess,Spot,patina)
    }

    pub(crate) fn get_texture(&mut self, geometry: &GeometryProgramName, element_id: &FlatId) -> Result<TextureDraw,Message> {
        let patina = self.shape_program(geometry,&PatinaProcessName::Texture(element_id.clone()))?.get_patina();
        force_branch!(PatinaProcess,Texture,patina)
    }

    pub(crate) fn get_free_texture(&mut self, geometry: &GeometryProgramName, element_id: &FlatId) -> Result<TextureDraw,Message> {
        let patina = self.shape_program(geometry,&PatinaProcessName::FreeTexture(element_id.clone()))?.get_patina();
        force_branch!(PatinaProcess,FreeTexture,patina)
    }

    pub(super) fn build(self, gl: &mut WebGlGlobal, canvases: &DrawingAllFlats) -> Result<Vec<Process>,Message> {
        let mut processes = vec![];
        for (character,mut prog) in self.store {
            match character {
                ProgramCharacter(_,PatinaProcessName::Texture(flat_id)) |
                ProgramCharacter(_,PatinaProcessName::FreeTexture(flat_id)) =>{
                    canvases.add_process(&flat_id,prog.get_process_mut())?;
                },
                _ => {}
            }
            processes.push(prog.into_process().build(gl,self.left)?);
        }
        Ok(processes)
    }
}
