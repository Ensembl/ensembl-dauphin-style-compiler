use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use crate::PgCommanderWeb;
use crate::webgl::{ProcessBuilder, ProgramBuilder, SourceInstrs};
use super::geometry::{GeometryProcessName, GeometryProgramName};
use super::patina::{PatinaProcessName, PatinaProgramName };
use crate::stage::stage::get_stage_source;
use crate::util::message::Message;
use enum_iterator::{Sequence, all};
use peregrine_toolkit::error::Error;
use peregrine_toolkit::{lock, log_extra};

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone,PartialEq,Eq,Hash,Sequence)]
pub(crate) struct WebGLProgramName(pub GeometryProgramName,pub PatinaProgramName);

impl WebGLProgramName {
    fn all_programs() -> impl Iterator<Item=WebGLProgramName> {
        all::<WebGLProgramName>()
    }
}

pub(crate) struct ProgramStoreEntry {
    builder: Rc<ProgramBuilder>
}

impl ProgramStoreEntry {
    fn new(builder: ProgramBuilder) -> Result<ProgramStoreEntry,Error> {
        Ok(ProgramStoreEntry {
            builder: Rc::new(builder)
        })
    }
    
    pub(crate) fn make_shape_program(&self, geometry_name: &GeometryProcessName, patina_name: &PatinaProcessName) -> Result<ProcessBuilder,Error> {
        let process = ProcessBuilder::new(self.builder.clone(),geometry_name,patina_name);
        Ok(process)
    }
}

struct ProgramStoreData {
    programs: HashMap<WebGLProgramName,ProgramStoreEntry>
}

impl ProgramStoreData {
    fn new() -> Result<ProgramStoreData,Message> {
        Ok(ProgramStoreData {
            programs: HashMap::new()
        })
    }

    fn make_program(&mut self, index: &WebGLProgramName) -> Result<(),Error> {
        let mut source = SourceInstrs::new(vec![]);
        source.merge(get_stage_source());
        source.merge(index.0.get_source());
        source.merge(index.1.get_source());
        let builder = ProgramBuilder::new(&source,index)?;
        self.programs.insert(index.clone(),ProgramStoreEntry::new(builder)?);
        Ok(())
    }

    fn get_program(&mut self, geometry: GeometryProgramName, patina: PatinaProgramName) -> Result<&ProgramStoreEntry,Error> {
        let index = WebGLProgramName(geometry,patina);
        if self.programs.get(&index).is_none() {
            self.make_program(&index)?;
        }
        Ok(self.programs.get(&index).as_ref().unwrap().clone())
    }
}

#[derive(Clone)]
pub struct ProgramStore(Arc<Mutex<ProgramStoreData>>);

impl ProgramStore {
    async fn async_background_load(&self) -> Result<(),Message> {
        for program in WebGLProgramName::all_programs() {
            lock!(self.0).get_program(program.0,program.1).ok(); // ok to discard result
        }
        log_extra!("program preloading done");
        Ok(())
    }

    fn background_load(&self, commander: &PgCommanderWeb) {
        let self2 = self.clone();
        commander.add("program preload", 10, None, None, Box::pin(async move {
            self2.async_background_load().await
        }));
    }

    pub(crate) fn new(commander: &PgCommanderWeb) -> Result<ProgramStore,Message> {
        let out = ProgramStore(Arc::new(Mutex::new(ProgramStoreData::new()?)));
        out.background_load(commander);
        Ok(out)
    }

    pub(super) fn get_shape_program(&self, geometry: &GeometryProcessName, patina: &PatinaProcessName) -> Result<ProcessBuilder,Error> {
        lock!(self.0).get_program(geometry.get_program_name(),patina.get_program_name())?.make_shape_program(geometry,patina)
    }
}
