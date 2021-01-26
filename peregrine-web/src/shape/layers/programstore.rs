use std::cell::RefCell;
use std::rc::Rc;
use crate::webgl::{ WebGlCompiler, Program, SourceInstrs };
use super::geometry::{ GeometryAccessorVariety, GeometryVarietyAccessor };
use super::patina::{ PatinaAccessorVariety, PatinaVarietyAccessor };

struct ProgramIndex(GeometryAccessorVariety,PatinaAccessorVariety);

impl ProgramIndex {
    const COUNT : usize = GeometryAccessorVariety::COUNT * PatinaAccessorVariety::COUNT;

    pub fn get_index(&self) -> usize {
        self.0.get_index() * PatinaAccessorVariety::COUNT + self.1.get_index()
    }
}

pub(crate) struct ProgramStoreEntry<'c> {
    program: Rc<Program<'c>>,
    geometry: GeometryVarietyAccessor,
    patina: PatinaVarietyAccessor
}

impl<'c> ProgramStoreEntry<'c> {
    fn new(program: Program<'c>, index: &ProgramIndex) -> anyhow::Result<ProgramStoreEntry<'c>> {
        let geometry = index.0.make_variety_accessor(&program)?;
        let patina = index.1.make_variety_accessor(&program)?;
        Ok(ProgramStoreEntry {
            program: Rc::new(program),
            geometry,
            patina
        })
    }

    pub(crate) fn program(&self) -> &Rc<Program<'c>> { &self.program }
    pub(crate) fn get_geometry(&self) -> &GeometryVarietyAccessor { &self.geometry }
    pub(crate) fn get_patina(&self) -> &PatinaVarietyAccessor { &self.patina }
}

pub struct ProgramStore<'c> {
    compiler: WebGlCompiler<'c>,
    programs: RefCell<[Option<Rc<ProgramStoreEntry<'c>>>;ProgramIndex::COUNT]>
}

impl<'c> ProgramStore<'c> {
    fn make_program(&self, index: &ProgramIndex) -> anyhow::Result<()> {
        let mut source = SourceInstrs::new(vec![]);
        source.merge(index.0.get_source());
        source.merge(index.1.get_source());
        self.programs.borrow_mut()[index.get_index()] = Some(Rc::new(ProgramStoreEntry::new(self.compiler.make_program(source)?,&index)?));
        Ok(())
    }

    pub(super) fn get_program(&self, geometry: GeometryAccessorVariety, patina: PatinaAccessorVariety) -> anyhow::Result<Rc<ProgramStoreEntry<'c>>> {
        let index = ProgramIndex(geometry,patina);
        if self.programs.borrow()[index.get_index()].is_none() {
            self.make_program(&index)?;
        }
        Ok(self.programs.borrow()[index.get_index()].as_ref().unwrap().clone())
    }
}