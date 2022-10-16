use crate::PeregrineCoreBase;
use crate::index::jumpstore::JumpStore;
use crate::shapeload::programloader::ProgramLoader;
use crate::{ StickStore, ShapeStore, DataStore };

#[derive(Clone)]
pub struct AgentStore {
    pub program_loader: ProgramLoader,
    pub stick_store: StickStore,
    pub jump_store: JumpStore,
    pub lane_store: ShapeStore,
    pub data_store: DataStore
}

impl AgentStore {
    pub fn new(base: &PeregrineCoreBase) -> AgentStore {
        let data_store = DataStore::new(10240,&base);
        let program_loader = ProgramLoader::new(&base);
        let stick_store = StickStore::new(&base);
        let lane_store = ShapeStore::new(4096,&base,&program_loader);
        let jump_store = JumpStore::new(&base);
        AgentStore {
            program_loader, stick_store, jump_store, lane_store, data_store
        }
    }
}
