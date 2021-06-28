use crate::webgl::{ ProcessBuilder };
use super::geometry::{ GeometryProgramName, GeometryProgramLink };
use super::patina::{ PatinaProcess, PatinaProcessName };

pub(crate) struct ShapeProgram {
    process: ProcessBuilder,
    geometry: GeometryProgramLink,
    patina: PatinaProcess
}

impl ShapeProgram {
    pub(super) fn new(process: ProcessBuilder, geometry: GeometryProgramLink, patina: PatinaProcess) -> ShapeProgram {
        ShapeProgram { process, geometry, patina }
    }

    pub(super) fn into_process(self) -> ProcessBuilder { self.process }
    pub(crate) fn get_process_mut(&mut self) -> &mut ProcessBuilder { &mut self.process }
    pub(crate) fn get_geometry(&self) -> &GeometryProgramLink { &self.geometry }
    pub(crate) fn get_patina(&self) -> &PatinaProcess { &self.patina }
}
