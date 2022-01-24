use peregrine_toolkit::lock;
use crate::{DataMessage, LaneStore, PeregrineCoreBase, PgCommanderTaskSpec, ShapeList, ShapeListBuilder, ShapeRequest, add_task, api::MessageSender, Scale};

#[derive(Clone)]
pub enum LoadMode {
    RealTime,
    Batch,
    Network
}

impl LoadMode {
    pub fn build_shapes(&self) -> bool {
        match self {
            LoadMode::Network => false,
            _ => true
        }
    }

    pub fn high_priority(&self) -> bool {
        match self {
            LoadMode::RealTime => true,
            _ => false
        }
    }
}

pub(crate) async fn load_shapes(base: &PeregrineCoreBase, result_store: &LaneStore, messages: Option<&MessageSender>, shape_requests: Vec<ShapeRequest>, scale: &Scale, mode: &LoadMode) -> (Option<ShapeList>,Vec<DataMessage>) {
    let mut errors = vec![];
    let lane_store = result_store.clone();
    let tracks : Vec<_> = shape_requests.iter().map(|request|{
        let request = request.clone();
        let mode = mode.clone();
        let lane_store = lane_store.clone();
        add_task(&base.commander,PgCommanderTaskSpec {
            name: format!("data program"),
            prio: if mode.high_priority() { 2 } else { 5 },
            slot: None,
            timeout: None,
            stats: false,
            task: Box::pin(async move {
                lane_store.run(&request,&mode).await.as_ref().clone()
            })
        })
    }).collect();
    if !mode.build_shapes() { return (None,errors); }
    let mut new_shapes = ShapeListBuilder::new(&base.allotment_metadata,&*lock!(base.assets));
    for future in tracks {
        future.finish_future().await;
        match future.take_result().as_ref().unwrap() {
            Ok(zoo) => {
                new_shapes.append(&zoo);
            },
            Err(e) => {
                if let Some(messages) = &messages {
                    messages.send(e.clone());
                }
                errors.push(e.clone());
            }
        }
    }
    (Some(ShapeList::new(new_shapes,Some(scale))),errors)
}