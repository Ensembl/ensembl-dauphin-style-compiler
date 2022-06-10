use crate::{DataMessage, ShapeStore, PeregrineCoreBase, PgCommanderTaskSpec, add_task, api::MessageSender,  shape::{AbstractShapesContainer}, ShapeRequestGroup, allotment::core::abstractcarriage::AbstractCarriage, CarriageExtent };

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

pub(crate) async fn load_carriage_shape_list(base: &PeregrineCoreBase, result_store: &ShapeStore, messages: Option<&MessageSender>, shape_requests: ShapeRequestGroup, extent: Option<&CarriageExtent>, mode: &LoadMode) -> Result<AbstractCarriage,Vec<DataMessage>> {
    let mut errors = vec![];
    let lane_store = result_store.clone();
    let tracks : Vec<_> = shape_requests.iter().map(|request|{
        let request = request.clone();
        let mode = mode.clone();
        let lane_store = lane_store.clone();
        add_task(&base.commander,PgCommanderTaskSpec {
            name: format!("data program {}",if mode.high_priority() { "high" } else { "low" }),
            prio: if mode.high_priority() { 2 } else { 5 },
            slot: None,
            timeout: None,
            stats: false,
            task: Box::pin(async move {
                lane_store.run(&request,&mode).await.as_ref().clone()
            })
        })
    }).collect();
    if !mode.build_shapes() { return Err(errors); }
    let mut new_shapes = vec![];
    for future in tracks {
        future.finish_future().await;
        match future.take_result().unwrap() {
            Ok(zoo) => {
                new_shapes.push(zoo.clone());
            },
            Err(e) => {
                if let Some(messages) = &messages {
                    messages.send(e.clone());
                }
                errors.push(e.clone());
            }
        }
    }
    let new_shapes = AbstractShapesContainer::merge(new_shapes);
    Ok(new_shapes.build_abstract_carriage(Some(&shape_requests),extent))
}
