use crate::{DataMessage, PeregrineCoreBase, PgCommanderTaskSpec, ProgramName, add_task, util::memoized::{Memoized, MemoizedType}};

fn make_program_loader(base: &PeregrineCoreBase) -> Memoized<ProgramName,Result<(),DataMessage>> {
    let base = base.clone();
    Memoized::new(MemoizedType::Store,move |_,program_name: &ProgramName| {
        let base = base.clone();
        let program_name = program_name.clone();
        Box::pin(async move { 
            let backend = base.all_backends.backend(&program_name.0)?;
            backend.program(&program_name).await?;
            if !base.dauphin.is_present(&program_name) {
                return Err(DataMessage::DauphinProgramDidNotLoad(program_name));
            }
            Ok(())
        })
    })
}

#[derive(Clone)]
pub struct ProgramLoader(Memoized<ProgramName,Result<(),DataMessage>>);

impl ProgramLoader {
    pub fn new(base: &PeregrineCoreBase) -> ProgramLoader {
        ProgramLoader(make_program_loader(base))
    }

    pub async fn load(&self, program_name: &ProgramName) -> Result<(),DataMessage> {
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
