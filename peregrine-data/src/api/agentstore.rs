use crate::PeregrineCoreBase;
use crate::index::jumpstore::JumpStore;
use crate::index::smallvaluesstore::SmallValuesStore;
use crate::{ StickStore, ShapeStore, DataStore };

#[derive(Clone)]
pub struct AgentStore {
    pub stick_store: StickStore,
    pub jump_store: JumpStore,
    pub lane_store: ShapeStore,
    pub data_store: DataStore,
    pub small_values_store: SmallValuesStore
}

impl AgentStore {
    pub fn new(base: &PeregrineCoreBase) -> AgentStore {
        let data_store = DataStore::new(10240,&base);
        let stick_store = StickStore::new(&base);
        let lane_store = ShapeStore::new(4096,&base);
        let jump_store = JumpStore::new(&base);
        let small_values_store = SmallValuesStore::new(&base);
        AgentStore {
            stick_store, jump_store, lane_store, data_store, small_values_store
        }
    }
}
