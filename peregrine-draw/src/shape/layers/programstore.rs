use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::webgl::{ProcessBuilder, ProgramBuilder, SourceInstrs};
use super::geometry::{GeometryProcessName, GeometryAdder, GeometryProgramName};
use super::patina::{PatinaProcessName, PatinaAdder, PatinaProgramName};
use super::shapeprogram::ShapeProgram;
use crate::stage::stage::get_stage_source;
use crate::util::message::Message;


#[derive(Clone,PartialEq,Eq,Hash)]
struct ProgramIndex(GeometryProgramName,PatinaProgramName);

pub(crate) struct ProgramStoreEntry {
    builder: Rc<ProgramBuilder>,
    geometry: GeometryAdder,
    patina: PatinaAdder
}

impl ProgramStoreEntry {
    fn new(builder: ProgramBuilder, index: &ProgramIndex) -> Result<ProgramStoreEntry,Message> {
        let geometry = index.0.make_geometry_program(&builder)?;
        let patina = index.1.make_patina_program(&builder)?;
        Ok(ProgramStoreEntry {
            builder: Rc::new(builder),
            geometry,
            patina
        })
    }
    
    pub(crate) fn make_shape_program(&self, geometry_process_name: &GeometryProcessName) -> Result<ShapeProgram,Message> {
        let geometry = self.geometry.clone();
        let patina = self.patina.make_patina_process()?;
        let process = ProcessBuilder::new(self.builder.clone());
        Ok(ShapeProgram::new(process,geometry,geometry_process_name.clone(),patina))
    }
}

struct ProgramStoreData {
    programs: HashMap<ProgramIndex,ProgramStoreEntry>
}

impl ProgramStoreData {
    fn new() -> Result<ProgramStoreData,Message> {
        Ok(ProgramStoreData {
            programs: HashMap::new()
        })
    }

    fn make_program(&mut self, index: &ProgramIndex) -> Result<(),Message> {
        let mut source = SourceInstrs::new(vec![]);
        source.merge(get_stage_source());
        source.merge(index.0.get_source());
        source.merge(index.1.get_source());
        let builder = ProgramBuilder::new(&source)?;
        self.programs.insert(index.clone(),ProgramStoreEntry::new(builder,&index)?);
        Ok(())
    }

    fn get_program(&mut self, geometry: GeometryProgramName, patina: PatinaProgramName) -> Result<&ProgramStoreEntry,Message> {
        let index = ProgramIndex(geometry,patina);
        if self.programs.get(&index).is_none() {
            self.make_program(&index)?;
        }
        Ok(self.programs.get(&index).as_ref().unwrap().clone())
    }
}

#[derive(Clone)]
pub struct ProgramStore(Rc<RefCell<ProgramStoreData>>);

impl ProgramStore {
    pub(crate) fn new() -> Result<ProgramStore,Message> {
        Ok(ProgramStore(Rc::new(RefCell::new(ProgramStoreData::new()?))))
    }

    pub(super) fn get_shape_program(&self, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> Result<ShapeProgram,Message> {
        self.0.borrow_mut().get_program(geometry.get_program_name(),patina.get_program_name())?.make_shape_program(geometry)
    }
}
