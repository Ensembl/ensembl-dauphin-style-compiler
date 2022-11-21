use std::sync::{Arc, Mutex};
use std::{collections::HashMap};
use commander::cdr_tick;
use peregrine_toolkit::log_extra;
use peregrine_toolkit_async::sync::retainer::RetainTest;

use crate::shape::layers::patina::PatinaProcess;
use crate::webgl::{ ProcessBuilder, Process, DrawingCanvases };
use super::geometry::{GeometryProcessName, GeometryYielder, TrianglesGeometry, GeometryAdder};
use super::programstore::ProgramStore;
use super::patina::{PatinaProcessName, PatinaYielder};
use crate::util::message::Message;
use crate::webgl::global::WebGlGlobal;
use super::shapeprogram::ShapeProgram;

/* 
TODO ensure + index
TODO y split bug
TODO layers from core
TODO ordered layers
TODO return shapes from core without cloning (drain)
TODO uniforms set only on change
TODO global destroy
TODO keep program when same program
TODO wiggle width
TODO intersection cache
*/

#[derive(Clone,PartialEq,Eq,Hash)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub(crate) struct ProgramCharacter(pub GeometryProcessName, pub PatinaProcessName);

impl ProgramCharacter {
    pub(crate) fn key(&self) -> String {
        format!("{}/{}",self.0.key(),self.1.get_program_name().key())
    }

    pub(crate) fn order(&self) -> (usize,usize,String) {
        (self.1.order(),0,self.key())
    }
}

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
        Ok(self.store.get_mut(&character).unwrap().get_process_mut())
    }

    pub(super) async fn build(mut self, gl: &Arc<Mutex<WebGlGlobal>>, canvases: &DrawingCanvases, retain: &RetainTest) -> Result<Option<Vec<Process>>,Message> {
        let mut processes = vec![];
        let mut characters = self.store.keys().cloned().collect::<Vec<_>>();
        characters.sort_by_cached_key(|c| c.order());
        for character in &characters {
            let mut prog = self.store.remove(&character).unwrap();
            match &character.0 {
                GeometryProcessName::Triangles(TrianglesGeometry::TrackingSpecial(use_vertical)) |
                GeometryProcessName::Triangles(TrianglesGeometry::Window(use_vertical)) => {
                    let draw = match prog.get_geometry() {
                        GeometryAdder::Triangles(adder) => Some(adder.clone()),
                        _ => None
                    };
                    if let Some(adder) = draw {
                        let process = prog.get_process_mut();
                        adder.set_use_vertical(process,if *use_vertical { 1.0 } else { 0.0 })?;
                    }
                },
                _ => {}
            }
            match &character.1 {
                PatinaProcessName::Texture(flat_id) |
                PatinaProcessName::FreeTexture(flat_id) =>{
                    canvases.add_process(&flat_id,prog.get_process_mut())?;
                },
                PatinaProcessName::Spot(colour) => {
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
            processes.push(prog.into_process().build(gl,self.left,character).await?);
            cdr_tick(0).await;
            if !retain.test() {
                log_extra!("dumped discarded drawing");
                return Ok(None);
            }
        }
        Ok(Some(processes))
    }
}
