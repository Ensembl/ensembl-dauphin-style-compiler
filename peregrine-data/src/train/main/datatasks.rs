use std::{sync::{Arc, Mutex}};
use peregrine_toolkit::lock;
use crate::{shapeload::{carriagebuilder::CarriageBuilder, loadshapes::LoadMode}, add_task, PgCommanderTaskSpec, async_complete_task, PeregrineCoreBase, ShapeStore, DataMessage, StickStore, train::model::trainextent::TrainExtent };

#[cfg(debug_trains)]
use peregrine_toolkit::log;

use super::train::StickData;

async fn load_one_carriage(base: &mut PeregrineCoreBase, shape_store: &ShapeStore, mut carriage: CarriageBuilder) -> Result<(),DataMessage> {
    carriage.load(base,&shape_store,LoadMode::RealTime).await
}

pub(crate) fn load_carriage(base: &mut PeregrineCoreBase, shape_store: &ShapeStore, builder: &CarriageBuilder) {
    let mut base2 = base.clone();
    let shape_store = shape_store.clone();
    let builder = builder.clone();
    let handle = add_task(&base.commander,PgCommanderTaskSpec {
        name: format!("carriage loader"),
        prio: 1,
        slot: None,
        timeout: None,
        task: Box::pin(async move {
            let result = load_one_carriage(&mut base2,&shape_store,builder.clone()).await;
            if let Err(e) = result {
                base2.messages.send(e.clone());
            }
            Ok(())
        }),
        stats: false
    });
    async_complete_task(&base.commander, &base.messages,handle,|e| (e,false));
}

async fn load_one_stick(base: &mut PeregrineCoreBase, stick_store: &StickStore, train_extent: &TrainExtent, stick_data: &Arc<Mutex<StickData>>) -> Result<(),DataMessage> {
    let output = stick_store.get(&train_extent.layout().stick()).await;
    let data = match output {
        Ok(value) => StickData::Ready(value),
        Err(e) => {
            base.messages.send(DataMessage::XXXTransitional(e.clone()));
            StickData::Unavailable
        }
    };
    *lock!(stick_data) = data;
    Ok(())
}

pub(crate) fn load_stick(base: &mut PeregrineCoreBase, stick_store: &StickStore, extent: &TrainExtent, output: &Arc<Mutex<StickData>>) {
    let mut base2 = base.clone();
    let stick_store = stick_store.clone();
    let extent = extent.clone();
    let output = output.clone();
    let handle = add_task(&base.commander,PgCommanderTaskSpec {
        name: format!("stick loader"),
        prio: 1,
        slot: None,
        timeout: None,
        task: Box::pin(async move {
            let result = load_one_stick(&mut base2,&stick_store,&extent,&output).await;
            if let Err(e) = result {
                base2.messages.send(e.clone());
            }
            Ok(())
        }),
        stats: false
    });
    async_complete_task(&base.commander, &base.messages,handle,|e| (e,false));
}
