use std::cell::RefCell;
use std::rc::Rc;
use crate::webgl::{ make_program, Program, SourceInstrs, GPUSpec, ProgramBuilder };
use super::geometry::{ GeometryProgramName, GeometryProgram };
use super::patina::{ PatinaProgramName, PatinaProgram };
use super::super::core::stage::get_stage_source;
use web_sys::WebGlRenderingContext;
use crate::util::message::Message;

struct ProgramIndex(GeometryProgramName,PatinaProgramName);

impl ProgramIndex {
    const COUNT : usize = GeometryProgramName::COUNT * PatinaProgramName::COUNT;

    pub fn get_index(&self) -> usize {
        self.0.get_index() * PatinaProgramName::COUNT + self.1.get_index()
    }
}

pub(crate) struct ProgramStoreEntry {
    builder: Rc<ProgramBuilder>,
    geometry: GeometryProgram,
    patina: PatinaProgram
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

    pub(crate) fn builder(&self) -> &Rc<ProgramBuilder> { &self.builder }
    pub(crate) fn get_geometry(&self) -> &GeometryProgram { &self.geometry }
    pub(crate) fn get_patina(&self) -> &PatinaProgram { &self.patina }
}

pub(crate) struct ProgramStoreData {
    programs: RefCell<Vec<Option<Rc<ProgramStoreEntry>>>>
}

impl ProgramStoreData {
    fn new() ->Result<ProgramStoreData,Message> {
        let programs = RefCell::new(vec![None;ProgramIndex::COUNT]);
        Ok(ProgramStoreData {
            programs
        })
    }

    fn make_program(&self, index: &ProgramIndex) -> Result<(),Message> {
        let mut source = SourceInstrs::new(vec![]);
        source.merge(get_stage_source());
        source.merge(index.0.get_source());
        source.merge(index.1.get_source());
        let builder = ProgramBuilder::new(&source)?;
        self.programs.borrow_mut()[index.get_index()] = Some(Rc::new(ProgramStoreEntry::new(builder,&index)?));
        Ok(())
    }

    pub(super) fn get_program(&self, geometry: GeometryProgramName, patina: PatinaProgramName) -> Result<Rc<ProgramStoreEntry>,Message> {
        let index = ProgramIndex(geometry,patina);
        if self.programs.borrow()[index.get_index()].is_none() {
            self.make_program(&index)?;
        }
        Ok(self.programs.borrow()[index.get_index()].as_ref().unwrap().clone())
    }
}

#[derive(Clone)]
pub struct ProgramStore(Rc<ProgramStoreData>);

impl ProgramStore {
    pub(crate) fn new() -> Result<ProgramStore,Message> {
        Ok(ProgramStore(Rc::new(ProgramStoreData::new()?)))
    }

    pub(super) fn get_program(&self, geometry: GeometryProgramName, patina: PatinaProgramName) -> Result<Rc<ProgramStoreEntry>,Message> {
        self.0.get_program(geometry,patina)
    }
}
