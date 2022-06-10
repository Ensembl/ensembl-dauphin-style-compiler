use crate::PeregrineCoreBase;
use crate::index::jumpstore::JumpStore;
use crate::shapeload::programloader::ProgramLoader;
use crate::{ AuthorityStore, StickStore, ShapeStore, DataStore };

#[derive(Clone)]
pub struct AgentStore {
    pub program_loader: ProgramLoader,
    pub stick_authority_store: AuthorityStore,
    pub stick_store: StickStore,
    pub jump_store: JumpStore,
    pub lane_store: ShapeStore,
    pub data_store: DataStore
}

impl AgentStore {
    pub fn new(base: &PeregrineCoreBase) -> AgentStore {
        /* Payloads are about 4k on the wire, maybe 4x that unpacked. 1000 => ~64Mb. */
        let data_store = DataStore::new(10240,&base);
        let program_loader = ProgramLoader::new(&base);
        let stick_authority_store = AuthorityStore::new(&base,&program_loader);
        let stick_store = StickStore::new(&base,&stick_authority_store);
        let lane_store = ShapeStore::new(4096,&base,&program_loader);
        let jump_store = JumpStore::new(&base,&stick_authority_store);
        AgentStore {
            program_loader, stick_authority_store, stick_store, jump_store, lane_store, data_store
        }
    }
}
