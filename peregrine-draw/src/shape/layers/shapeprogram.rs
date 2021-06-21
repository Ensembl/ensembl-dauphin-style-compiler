use crate::webgl::{ ProcessBuilder };
use super::geometry::{ GeometryProgramName, GeometryProgram };
use super::programstore::ProgramStore;
use super::patina::{ PatinaProcess, PatinaProcessName };
use crate::util::message::Message;

pub(crate) struct ShapeProgram {
    process: ProcessBuilder,
    geometry: GeometryProgram,
    patina: PatinaProcess
}

impl ShapeProgram {
    pub(super) fn new(process: ProcessBuilder, geometry: GeometryProgram, patina: PatinaProcess) -> ShapeProgram {
        ShapeProgram { process, geometry, patina }
    }

    pub(super) fn into_process(self) -> ProcessBuilder { self.process }
    pub(crate) fn get_process_mut(&mut self) -> &mut ProcessBuilder { &mut self.process }
    pub(crate) fn get_geometry(&self) -> &GeometryProgram { &self.geometry }
    pub(crate) fn get_patina(&self) -> &PatinaProcess { &self.patina }
}
