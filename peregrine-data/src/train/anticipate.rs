use std::{sync::{Arc, Mutex}};
use commander::CommanderStream;
use peregrine_toolkit::sync::needed::Needed;
use crate::{Carriage, CarriageExtent, DataMessage, LaneStore, PeregrineCoreBase, PgCommanderTaskSpec, Scale, add_task, core::Layout, lane::shapeloader::LoadMode, switch::trackconfiglist::TrainTrackConfigList };
use super::{carriage::CarriageSerialSource, trainextent::TrainExtent};

struct AnticipateTask {
    carriages: Vec<Carriage>,
    batch: bool
}

impl AnticipateTask {
    fn new(carriages: Vec<Carriage>, batch: bool) -> AnticipateTask {
        AnticipateTask { carriages, batch }
    }

    async fn run(&mut self, base: &PeregrineCoreBase, result_store: &LaneStore) -> Result<(),DataMessage> {
        let mut handles = vec![];
        let load_mode = if self.batch { LoadMode::Network } else { LoadMode::Batch };
        for mut carriage in self.carriages.drain(..) {
            let load_mode = load_mode.clone();
            handles.push(async move {
                carriage.load(&base,&result_store,load_mode.clone()).await
            });
        }
        for handle in handles {
            handle.await?;
        }
        Ok(())
    }
}

fn run_anticipator(base: &PeregrineCoreBase, result_store: &LaneStore, stream: &CommanderStream<AnticipateTask>) {
    let stream = stream.clone();
    let base2 = base.clone();
    let result_store = result_store.clone();
    add_task::<()>(&base.commander,PgCommanderTaskSpec {
        name: format!("anticipator"),
        prio: 9,
        slot: None,
        timeout: None,
        stats: false,
        task: Box::pin(async move {
            loop {
                stream.get().await.run(&base2,&result_store).await?;
            }
        })
    });
}

pub struct Anticipate {
    try_lifecycle: Needed,
    serial_source: CarriageSerialSource,
    extent: Arc<Mutex<Option<CarriageExtent>>>,
    stream: CommanderStream<AnticipateTask>
}

impl Anticipate {
    pub(crate) fn new(base: &PeregrineCoreBase, try_lifecycle: &Needed, result_store: &LaneStore, serial_source: &CarriageSerialSource) -> Anticipate {
        let stream = CommanderStream::new();
        run_anticipator(&base,&result_store,&stream);
        Anticipate {
            try_lifecycle: try_lifecycle.clone(),
            serial_source: serial_source.clone(),
            extent: Arc::new(Mutex::new(None)),
            stream
        }
    }

    fn lightweight(&self) -> bool {
        cfg!(debug_assertions)
    }

    fn build_carriage(&self, carriages: &mut Vec<Carriage>, layout: &Layout, scale: &Scale, index: i64) {
        if index < 0 { return; }
        let train_track_config_list = TrainTrackConfigList::new(layout,scale); // TODO cache
        let train_extent = TrainExtent::new(layout,scale);
        let carriage_extent = CarriageExtent::new(&train_extent,index as u64);
        let carriage = Carriage::new(&self.try_lifecycle,&self.serial_source,&carriage_extent,&train_track_config_list,None,true);
        carriages.push(carriage);
    }

    fn build_carriages(&self, layout: &Layout, extent: &CarriageExtent, amount: i64) -> Result<Vec<Carriage>,DataMessage> {
        let mut carriages = vec![];
        let width = 6;
        let base_index = extent.index();
        for offset in -width..(width+1) {
            for delta in 0..amount {
                /* out */
                let new_scale = extent.train().scale().delta_scale(delta);
                if let Some(new_scale) = &new_scale {
                    let new_base_index = new_scale.convert_index(extent.train().scale(),base_index) as i64;
                    self.build_carriage(&mut carriages,layout,new_scale,new_base_index+offset);
                }
                /* in */
                let new_scale = extent.train().scale().delta_scale(-delta);
                if let Some(new_scale) = &new_scale {
                    let new_base_index = new_scale.convert_index(extent.train().scale(),base_index) as i64;
                    self.build_carriage(&mut carriages,layout,new_scale,new_base_index+offset);
                }
            }
        }
        //
        Ok(carriages)
    }


    fn build_tasks(&self, extent: &CarriageExtent, amount: i64, network_only: bool) -> Result<(),DataMessage> {
        let layout = extent.train().layout().clone();
        let carriages = self.build_carriages(&layout,extent,amount)?;
        if network_only {
            self.stream.add(AnticipateTask::new(carriages,true));
        } else {
            for carriage in carriages {
                self.stream.add(AnticipateTask::new(vec![carriage],false));
            }
        }
        Ok(())
    }

    pub(crate) fn anticipate(&self, extent: &CarriageExtent) -> Result<(),DataMessage> {
        if let Some(old_extent) = self.extent.lock().unwrap().as_ref() {
            if extent == old_extent { return Ok(()); }
        }
        self.stream.clear();
        if self.lightweight() {
            self.build_tasks(extent,2,false)?;
        } else {
            self.build_tasks(extent,4,true)?;
            self.build_tasks(extent,4,false)?;
            self.build_tasks(extent,20,true)?;
            self.build_tasks(extent,20,false)?;
        }
        *self.extent.lock().unwrap() = Some(extent.clone());
        Ok(())
    }
}
