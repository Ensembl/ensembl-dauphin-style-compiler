use std::cell::RefCell;
use std::rc::Rc;
use crate::webgl::{ WebGlCompiler, Program, SourceInstrs };
use super::geometry::GeometryAccessorVariety;
use super::patina::PatinaAccessorVariety;

struct ProgramIndex(GeometryAccessorVariety,PatinaAccessorVariety);

impl ProgramIndex {
    const COUNT : usize = GeometryAccessorVariety::COUNT * PatinaAccessorVariety::COUNT;

    pub fn get_index(&self) -> usize {
        self.0.get_index() * PatinaAccessorVariety::COUNT + self.1.get_index()
    }
}

pub struct ProgramStore<'c> {
    compiler: WebGlCompiler<'c>,
    programs: RefCell<[Option<Rc<Program<'c>>>;ProgramIndex::COUNT]>
}

impl<'c> ProgramStore<'c> {
    fn make_program(&self, index: &ProgramIndex) -> anyhow::Result<()> {
        let mut source = SourceInstrs::new(vec![]);
        source.merge(index.0.get_source());
        source.merge(index.1.get_source());
        self.programs.borrow_mut()[index.get_index()] = Some(Rc::new(self.compiler.make_program(source)?));
        Ok(())
    }

    pub(super) fn get_program(&self, geometry: GeometryAccessorVariety, patina: PatinaAccessorVariety) -> anyhow::Result<Rc<Program<'c>>> {
        let index = ProgramIndex(geometry,patina);
        if self.programs.borrow()[index.get_index()].is_none() {
            self.make_program(&index)?;
        }
        Ok(self.programs.borrow()[index.get_index()].as_ref().unwrap().clone())
    }
}