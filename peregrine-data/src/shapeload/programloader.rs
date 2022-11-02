use peregrine_toolkit::error::Error;
use crate::{PeregrineCoreBase, PgCommanderTaskSpec, add_task, util::memoized::{Memoized, MemoizedType}};
use super::programname::ProgramName;

async fn get_program(base: &PeregrineCoreBase, program_name: &ProgramName) -> Result<(),Error> {
    for backend_namespace in &base.channel_registry.all() {
        let backend = base.all_backends.backend(backend_namespace)?;
        backend.program(program_name).await?;
        if base.dauphin.is_present(&program_name) { break; }
    }
    Ok(())
}

fn make_program_loader(base: &PeregrineCoreBase) -> Memoized<ProgramName,Result<(),Error>> {
    let base = base.clone();
    Memoized::new(MemoizedType::Store,move |_,program_name: &ProgramName| {
        let base = base.clone();
        let program_name = program_name.clone();
        Box::pin(async move { 
            get_program(&base,&program_name).await?;
            if !base.dauphin.is_present(&program_name) {
                return Err(Error::operr(&format!("program did not load: {:?}",program_name)));
            }
            Ok(())
        })
    })
}

#[derive(Clone)]
pub struct ProgramLoader(Memoized<ProgramName,Result<(),Error>>);

impl ProgramLoader {
    pub fn new(base: &PeregrineCoreBase) -> ProgramLoader {
        ProgramLoader(make_program_loader(base))
    }

    pub async fn load(&self, program_name: &ProgramName) -> Result<(),Error> {
        self.0.get(program_name).await.as_ref().clone()
    }

    pub fn load_background(&self, base: &PeregrineCoreBase, program_name: &ProgramName) {
        let cache = self.0.clone();
        let program_name = program_name.clone();
        add_task(&base.commander,PgCommanderTaskSpec {
            name: format!("program background loader"),
            prio: 10,
            slot: None,
            timeout: None,
            task: Box::pin(async move {
                cache.get(&program_name).await;
                Ok(())
            }),
            stats: false
        });
    }
}
