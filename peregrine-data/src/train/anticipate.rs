use std::{sync::{Arc, Mutex}};
use commander::CommanderStream;
use peregrine_toolkit::sync::needed::Needed;
use crate::{Carriage, CarriageExtent, DataMessage, ShapeStore, PeregrineCoreBase, PgCommanderTaskSpec, Scale, add_task, core::{Layout, pixelsize::PixelSize}, lane::shapeloader::LoadMode, switch::trackconfiglist::TrainTrackConfigList };
use super::{carriage::CarriageSerialSource, trainextent::TrainExtent};

struct AnticipateTask {
    carriages: Vec<Carriage>,
    batch: bool
}

impl AnticipateTask {
    fn new(carriages: Vec<Carriage>, batch: bool) -> AnticipateTask {
        AnticipateTask { carriages, batch }
    }

    async fn run(&mut self, base: &PeregrineCoreBase, result_store: &ShapeStore) -> Result<(),DataMessage> {
        let mut handles = vec![];
        let load_mode = if self.batch { LoadMode::Network } else { LoadMode::Batch };
        for mut carriage in self.carriages.drain(..) {
            let load_mode = load_mode.clone();
            let result_store = result_store.clone();
            let base2 = base.clone();
            let handle = add_task(&base.commander,PgCommanderTaskSpec {
                name: format!("data program"),
                prio: 9,
                slot: None,
                timeout: None,
                stats: false,
                task: Box::pin(async move {
                    carriage.load(&base2,&result_store,load_mode.clone()).await
                })
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.finish_future().await;
        }
        Ok(())
    }
}

fn run_anticipator(base: &PeregrineCoreBase, result_store: &ShapeStore, stream: &CommanderStream<AnticipateTask>) {
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
    pub(crate) fn new(base: &PeregrineCoreBase, try_lifecycle: &Needed, result_store: &ShapeStore, serial_source: &CarriageSerialSource) -> Anticipate {
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

    fn build_carriage(&self, carriages: &mut Vec<Carriage>, layout: &Layout, scale: &Scale, pixel_size: &PixelSize, index: i64) {
        if index < 0 { return; }
        let train_track_config_list = TrainTrackConfigList::new(layout,scale); // TODO cache
        let train_extent = TrainExtent::new(layout,scale,pixel_size);
        let carriage_extent = CarriageExtent::new(&train_extent,index as u64);
        let carriage = Carriage::new(&self.try_lifecycle,&self.serial_source,&carriage_extent,&train_track_config_list,None,true);
        carriages.push(carriage);
    }

    fn build_carriages(&self, layout: &Layout, extent: &CarriageExtent, amount_depth: i64, amount_width: i64) -> Result<Vec<Carriage>,DataMessage> {
        let mut carriages = vec![];
        let base_index = extent.index();
        for offset in -amount_width..(amount_width+1) {
            for delta in 0..amount_depth {
                /* out */
                let new_scale = extent.train().scale().delta_scale(delta);
                if let Some(new_scale) = &new_scale {
                    let new_base_index = new_scale.convert_index(extent.train().scale(),base_index) as i64;
                    self.build_carriage(&mut carriages,layout,new_scale,extent.train().pixel_size(),new_base_index+offset);
                }
                /* in */
                let new_scale = extent.train().scale().delta_scale(-delta);
                if let Some(new_scale) = &new_scale {
                    let new_base_index = new_scale.convert_index(extent.train().scale(),base_index) as i64;
                    self.build_carriage(&mut carriages,layout,new_scale,extent.train().pixel_size(),new_base_index+offset);
                }
            }
        }
        Ok(carriages)
    }


    fn build_tasks(&self, extent: &CarriageExtent, amount_depth: i64, amount_width: i64, network_only: bool) -> Result<(),DataMessage> {
        let layout = extent.train().layout().clone();
        let carriages = self.build_carriages(&layout,extent,amount_depth,amount_width)?;
        if network_only {
            self.stream.add(AnticipateTask::new(carriages,true));
        } else {
            self.stream.add(AnticipateTask::new(carriages,false));
        }
        Ok(())
    }

    pub(crate) fn anticipate(&self, extent: &CarriageExtent) -> Result<(),DataMessage> {
        if let Some(old_extent) = self.extent.lock().unwrap().as_ref() {
            if extent == old_extent { return Ok(()); }
        }
        self.stream.clear();
        if self.lightweight() {
            //self.build_tasks(extent,2,2,false)?;
        } else {
            self.build_tasks(extent,8,0,true)?;
            self.build_tasks(extent,8,0,false)?;
            self.build_tasks(extent,8,2,false)?;
            self.build_tasks(extent,8,6,false)?;
            self.build_tasks(extent,30,2,false)?;
            self.build_tasks(extent,30,6,false)?;
        }
        *self.extent.lock().unwrap() = Some(extent.clone());
        Ok(())
    }
}
