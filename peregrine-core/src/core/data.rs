use crate::PgCommander;
use crate::index::StickStore;
use crate::panel::PanelStore;
use crate::core::{ Viewport };
use crate::train::{ TrainSet };
use crate::api::PeregrineIntegration;
use std::sync::{ Arc, Mutex };

#[derive(Clone)]
pub struct PeregrineData {
    pub commander: PgCommander,
    pub panel_store: PanelStore,
    pub integration: Arc<Mutex<Box<dyn PeregrineIntegration>>>,
    pub train_set: TrainSet,
    pub stick_store: StickStore,
    pub viewport: Viewport
}
