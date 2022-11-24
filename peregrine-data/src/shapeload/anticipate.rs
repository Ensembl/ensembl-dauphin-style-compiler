use std::{sync::{Arc, Mutex}};
use commander::{CommanderStream, cdr_timer};
use peregrine_toolkit::{log_extra, error::Error};
use crate::{DataMessage, ShapeStore, PeregrineCoreBase, PgCommanderTaskSpec, Scale, add_task, core::{Layout, pixelsize::PixelSize}, shapeload::loadshapes::LoadMode, switch::trackconfiglist::TrainTrackConfigList, CarriageExtent, train::model::trainextent::TrainExtent, PeregrineApiQueue };
use crate::shapeload::carriagebuilder::CarriageBuilder;

const LIGHTWEIGHT_SCHEDULE : &[(i64,i64,bool)] = &[
    (2,0,true),(-2,0,true),   /* slight zoom in and out at this position, network only */
    (2,2,false),(-2,2,false)  /* slight zoom in and out, slight flank */
];
const HEAVYWEIGHT_SCHEDULE : &[(i64,i64,bool)]= &[
    (8,0,true),(-8,2,true),   /* slight zoom in and out at and near this position, network only */
    (30,0,true),              /* all zoomed out to origin, network only (looks bad and good hit rate) */
    (8,2,false),(-8,2,false), /* slight zoom in and out at and near this position */
    (30,0,false),             /* all zoomed out to origin */
    (8,2,false),(-8,6,false), /* around here with significant flank */
    (-30,2,true),(30,2,true), /* all scales with slight flank  */
    (-30,6,true)];            /* all scales iwith significant flank, network only  */

    fn lightweight() -> bool {
        cfg!(debug_assertions)
    }

struct AnticipateTask {
    carriages: Vec<CarriageBuilder>,
    batch: bool
}

impl AnticipateTask {
    fn new(carriages: Vec<CarriageBuilder>, batch: bool) -> AnticipateTask {
        AnticipateTask { carriages, batch }
    }

    async fn run(&mut self, base: &PeregrineCoreBase, result_store: &ShapeStore) -> Result<(),Error> {
        let mut handles = vec![];
        let load_mode = if self.batch { LoadMode::Network } else { LoadMode::Batch };
        for mut carriage in self.carriages.drain(..) {
            let load_mode = load_mode.clone();
            let result_store = result_store.clone();
            let base2 = base.clone();
            let handle = add_task(&base.commander,PgCommanderTaskSpec {
                name: format!("anticipator"),
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
    let stream2 = stream.clone();
    base2.shutdown.add(move || {
        stream2.add(AnticipateTask {
            carriages: vec![],
            batch: true
        });
    });
    let pause = if lightweight() { 2000. } else { 100. };
    add_task::<()>(&base.commander,PgCommanderTaskSpec {
        name: format!("anticipator"),
        prio: 9,
        slot: None,
        timeout: None,
        stats: false,
        task: Box::pin(async move {
            while !base2.shutdown.poll() {
                stream.get().await.run(&base2,&result_store).await?;
                cdr_timer(pause).await;
            }
            log_extra!("anticipator finishing");
            Ok(())
        })
    });
}

pub struct Anticipate {
    api_queue: PeregrineApiQueue,
    extent: Arc<Mutex<Option<CarriageExtent>>>,
    stream: CommanderStream<AnticipateTask>
}

impl Anticipate {
    pub(crate) fn new(base: &PeregrineCoreBase, result_store: &ShapeStore) -> Anticipate {
        let stream = CommanderStream::new();
        run_anticipator(&base,&result_store,&stream);
        Anticipate {
            api_queue: base.queue.clone(),
            extent: Arc::new(Mutex::new(None)),
            stream
        }
    }

    fn enabled(&self) -> bool {
        !cfg!(disable_anticipate)
    }

    fn build_carriage(&self, carriages: &mut Vec<CarriageBuilder>, layout: &Layout, scale: &Scale, pixel_size: &PixelSize, index: i64) {
        if index < 0 { return; }
        let train_track_config_list = TrainTrackConfigList::new(layout,scale); // TODO cache
        let train_extent = TrainExtent::new(layout,scale,pixel_size);
        let carriage_extent = CarriageExtent::new(&train_extent,index as u64);
        let carriage = CarriageBuilder::new(&self.api_queue,&carriage_extent,&train_track_config_list,None,true);
        carriages.push(carriage);
    }

    fn build_carriages(&self, layout: &Layout, extent: &CarriageExtent, amount_depth: i64, amount_width: i64) -> Result<Vec<CarriageBuilder>,DataMessage> {
        let mut carriages = vec![];
        let base_index = extent.index();
        for offset in -amount_width..(amount_width+1) {
            for delta in 0..amount_depth.abs() {
                let new_scale = extent.train().scale().delta_scale(delta*amount_depth.signum());
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
        if self.enabled() {
            let schedule = if lightweight() {
                LIGHTWEIGHT_SCHEDULE
            } else {
                HEAVYWEIGHT_SCHEDULE
            };
            for (depth,width,net_only) in schedule {
                self.build_tasks(extent,*depth,*width,*net_only)?;
            }
        }
        *self.extent.lock().unwrap() = Some(extent.clone());
        Ok(())
    }
}
