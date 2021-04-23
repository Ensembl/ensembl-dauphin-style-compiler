use crate::webgl::{ ProcessBuilder };
use super::geometry::{ GeometryProgramName, GeometryProgram };
use super::programstore::ProgramStore;
use super::patina::{ PatinaProcess, PatinaProcessName };
use crate::util::message::Message;

pub(super) struct ShapeProgram {
    process: ProcessBuilder,
    geometry: GeometryProgram,
    patina: PatinaProcess
}

impl ShapeProgram {
    pub(super) fn new(programs: &ProgramStore, geometry_program_name: &GeometryProgramName, patina_process_name: &PatinaProcessName) -> Result<ShapeProgram,Message> {
        let patina_program_name = patina_process_name.get_program_name();
        let program_store_entry = programs.get_program(geometry_program_name.clone(),patina_program_name)?;
        let geometry = program_store_entry.get_geometry().clone();
        let patina = program_store_entry.get_patina().make_patina_process(patina_process_name)?;
        let process = ProcessBuilder::new(program_store_entry.builder().clone());
        Ok(ShapeProgram {
            process,
            geometry,
            patina
        })
    }

    pub(super) fn into_process(self) -> ProcessBuilder { self.process }
    pub(super) fn get_process_mut(&mut self) -> &mut ProcessBuilder { &mut self.process }
    pub(super) fn get_geometry(&self) -> &GeometryProgram { &self.geometry }
    pub(super) fn get_patina(&self) -> &PatinaProcess { &self.patina }
}
