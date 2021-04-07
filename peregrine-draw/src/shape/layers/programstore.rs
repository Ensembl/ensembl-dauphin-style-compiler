use std::cell::RefCell;
use std::rc::Rc;
use crate::webgl::{ make_program, Program, SourceInstrs, GPUSpec };
use super::geometry::{ GeometryProgramName, GeometryProgram };
use super::patina::{ PatinaProgramName, PatinaProgram };
use super::super::core::stage::get_stage_source;
use web_sys::WebGlRenderingContext;
use crate::util::message::Message;
use crate::webgl::ProtoProgram;

struct ProgramIndex(GeometryProgramName,PatinaProgramName);

impl ProgramIndex {
    const COUNT : usize = GeometryProgramName::COUNT * PatinaProgramName::COUNT;

    pub fn get_index(&self) -> usize {
        self.0.get_index() * PatinaProgramName::COUNT + self.1.get_index()
    }
}

pub(crate) struct ProgramStoreEntry {
    program: Rc<Program>,
    geometry: GeometryProgram,
    patina: PatinaProgram
}

impl ProgramStoreEntry {
    fn new(program: Program, index: &ProgramIndex) -> Result<ProgramStoreEntry,Message> {
        let geometry = index.0.make_geometry_program(&program)?;
        let patina = index.1.make_patina_program(&program)?;
        Ok(ProgramStoreEntry {
            program: Rc::new(program),
            geometry,
            patina
        })
    }

    pub(crate) fn program(&self) -> &Rc<Program> { &self.program }
    pub(crate) fn get_geometry(&self) -> &GeometryProgram { &self.geometry }
    pub(crate) fn get_patina(&self) -> &PatinaProgram { &self.patina }
}

pub(crate) struct ProgramStoreData {
    gpu_spec: GPUSpec,
    programs: RefCell<Vec<Option<Rc<ProgramStoreEntry>>>>
}

impl ProgramStoreData {
    fn new(context: &WebGlRenderingContext) ->Result<ProgramStoreData,Message> {
        let gpuspec = GPUSpec::new(context)?;
        let programs = RefCell::new(vec![None;ProgramIndex::COUNT]);
        Ok(ProgramStoreData {
            gpu_spec: gpuspec,
            programs
        })
    }

    fn make_program(&self, context: &WebGlRenderingContext, gpuspec: &GPUSpec, index: &ProgramIndex) -> Result<(),Message> {
        let mut source = SourceInstrs::new(vec![]);
        source.merge(get_stage_source());
        source.merge(index.0.get_source());
        source.merge(index.1.get_source());
        let proto = ProtoProgram::new(source)?;
        self.programs.borrow_mut()[index.get_index()] = Some(Rc::new(ProgramStoreEntry::new(proto.make(context,gpuspec)?,&index)?));
        Ok(())
    }

    pub(super) fn get_program(&self,context: &WebGlRenderingContext, gpuspec: &GPUSpec, geometry: GeometryProgramName, patina: PatinaProgramName) -> Result<Rc<ProgramStoreEntry>,Message> {
        let index = ProgramIndex(geometry,patina);
        if self.programs.borrow()[index.get_index()].is_none() {
            self.make_program(context,gpuspec,&index)?;
        }
        Ok(self.programs.borrow()[index.get_index()].as_ref().unwrap().clone())
    }

    pub(super) fn gpu_spec(&self) -> &GPUSpec { &self.gpu_spec }
}

#[derive(Clone)]
pub struct ProgramStore(Rc<ProgramStoreData>);

impl ProgramStore {
    pub(crate) fn new(context: &WebGlRenderingContext) -> Result<ProgramStore,Message> {
        Ok(ProgramStore(Rc::new(ProgramStoreData::new(context)?)))
    }

    pub(super) fn get_program(&self, context: &WebGlRenderingContext, gpuspec: &GPUSpec, geometry: GeometryProgramName, patina: PatinaProgramName) -> Result<Rc<ProgramStoreEntry>,Message> {
        self.0.get_program(context,gpuspec,geometry,patina)
    }

    pub(crate) fn gpu_spec(&self) -> &GPUSpec { self.0.gpu_spec() }
}
