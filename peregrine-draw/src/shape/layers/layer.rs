use std::{collections::HashMap};
use crate::shape::layers::patina::PatinaProcess;
use crate::webgl::{ ProcessBuilder, Process, DrawingAllFlats };
use super::geometry::{GeometryProcessName, GeometryYielder};
use super::programstore::ProgramStore;
use super::patina::{PatinaProcessName, PatinaYielder};
use crate::util::message::Message;
use crate::webgl::global::WebGlGlobal;
use super::shapeprogram::ShapeProgram;

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

#[derive(Clone,Debug,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub(crate) struct ProgramCharacter(pub GeometryProcessName, pub PatinaProcessName);

pub(crate) struct Layer {
    programs: ProgramStore,
    store: HashMap<ProgramCharacter,ShapeProgram>,
    left: f64
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

    fn shape_program(&mut self, character: &ProgramCharacter) -> Result<&mut ShapeProgram,Message> {
        if !self.store.contains_key(&character) {
            self.store.insert(character.clone(),self.programs.get_shape_program(&character.0,&character.1)?);
        }
        Ok(self.store.get_mut(&character).unwrap())
    }

    pub(crate) fn get_process_builder(&mut self, geometry: &mut GeometryYielder, patina: &mut dyn PatinaYielder) -> Result<&mut ProcessBuilder,Message> {
        let geometry_name = geometry.name();
        let patina_name = patina.name();
        let character = ProgramCharacter(geometry_name.clone(),patina_name.clone());
        let shape_program = self.shape_program(&character)?; 
        geometry.set(shape_program.get_geometry())?;
        patina.set(shape_program.get_patina())?;
        let adder = shape_program.get_geometry().clone();
        shape_program.get_geometry_process_name().clone().apply_to_process(&adder,shape_program.get_process_mut())?;
        Ok(self.store.get_mut(&character).unwrap().get_process_mut())
    }

    pub(super) fn build(mut self, gl: &mut WebGlGlobal, canvases: &DrawingAllFlats) -> Result<Vec<Process>,Message> {
        let mut processes = vec![];
        let mut characters = self.store.keys().cloned().collect::<Vec<_>>();
        characters.sort();
        for character in &characters {
            let mut prog = self.store.remove(&character).unwrap();
            match character {
                ProgramCharacter(_,PatinaProcessName::Texture(flat_id)) |
                ProgramCharacter(_,PatinaProcessName::FreeTexture(flat_id)) =>{
                    canvases.add_process(&flat_id,prog.get_process_mut())?;
                },
                ProgramCharacter(_,PatinaProcessName::Spot(colour)) => {
                    let draw = match prog.get_patina() {
                        PatinaProcess::Spot(draw) => Some(draw),
                        _ => None
                    }.cloned();
                    if let Some(draw) = draw {
                        let process = prog.get_process_mut();
                        draw.set_spot(process,colour)?;
                    }
                },
                _ => {}
            }
            processes.push(prog.into_process().build(gl,self.left)?);
        }
        Ok(processes)
    }
}
