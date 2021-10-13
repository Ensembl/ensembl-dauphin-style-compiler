use crate::webgl::{ ProcessBuilder };
use super::geometry::{GeometryAdder, GeometryProcessName};
use super::patina::{ PatinaProcess };

pub(crate) struct ShapeProgram {
    process: ProcessBuilder,
    geometry: GeometryAdder,
    geometry_process_name: GeometryProcessName,
    patina: PatinaProcess
}

impl ShapeProgram {
    pub(super) fn new(process: ProcessBuilder, geometry: GeometryAdder, geometry_process_name: GeometryProcessName, patina: PatinaProcess) -> ShapeProgram {
        ShapeProgram { process, geometry, patina, geometry_process_name }
    }

    pub(super) fn into_process(self) -> ProcessBuilder { self.process }
    pub(crate) fn get_process_mut(&mut self) -> &mut ProcessBuilder { &mut self.process }
    pub(crate) fn get_geometry(&self) -> &GeometryAdder { &self.geometry }
    pub(crate) fn get_geometry_process_name(&self) -> &GeometryProcessName { &self.geometry_process_name }
    pub(crate) fn get_patina(&self) -> &PatinaProcess { &self.patina }
}
