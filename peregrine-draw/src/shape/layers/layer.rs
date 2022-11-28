use std::sync::{Arc, Mutex};
use std::{collections::HashMap};
use commander::cdr_tick;
use peregrine_toolkit::error::Error;
use peregrine_toolkit::log_extra;
use peregrine_toolkit_async::sync::retainer::RetainTest;
use crate::webgl::{ ProcessBuilder, Process };
use super::geometry::{GeometryProcessName, GeometryFactory};
use super::programstore::ProgramStore;
use super::patina::{PatinaProcessName, PatinaFactory};
use crate::webgl::global::WebGlGlobal;

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
    store: HashMap<ProgramCharacter,ProcessBuilder>,
    left: f64
}

impl Layer {
    pub fn new(programs: &ProgramStore, left: f64) -> Result<Layer,Error> {
        Ok(Layer {
            programs: programs.clone(),
            store: HashMap::new(),
            left
        })
    }

    fn shape_program(&mut self, character: &ProgramCharacter) -> Result<&mut ProcessBuilder,Error> {
        if !self.store.contains_key(&character) {
            self.store.insert(character.clone(),self.programs.get_shape_program(&character.0,&character.1)?);
        }
        Ok(self.store.get_mut(&character).unwrap())
    }

    pub(crate) fn get_process_builder(&mut self, geometry_factory: &dyn GeometryFactory, patina_factory: &dyn PatinaFactory) -> Result<&mut ProcessBuilder,Error> {
        let character = ProgramCharacter(geometry_factory.geometry_name(),patina_factory.patina_name());
        let process = self.shape_program(&character)?; 
        Ok(process)
    }

    pub(super) async fn build(mut self, gl: &Arc<Mutex<WebGlGlobal>>, retain: &RetainTest) -> Result<Option<Vec<Process>>,Error> {
        let mut processes = vec![];
        let mut characters = self.store.keys().cloned().collect::<Vec<_>>();
        characters.sort_by_cached_key(|c| c.order());
        for character in &characters {
            let process = self.store.remove(&character).unwrap();
            processes.push(process.build(gl,self.left,character).await?);
            cdr_tick(0).await;
            if !retain.test() {
                log_extra!("dumped discarded drawing");
                return Ok(None);
            }
        }
        Ok(Some(processes))
    }
}
